import pandas as pd
import psycopg2
import argparse
import sys

def main():
    parser = argparse.ArgumentParser(description='Data importer arguments.')
    parser.add_argument("--user", default="postgres")
    parser.add_argument("--password")
    parser.add_argument("--host", default="localhost")
    parser.add_argument("--port", default=5432, type=int)
    parser.add_argument("--reset-db", action="store_true")
    parser.add_argument("--dump-hdf", action="store_true")
    parser.add_argument("--source-tsv")

    args = parser.parse_args()
    cursor = None
    if(args.password):
        db_conn = psycopg2.connect(dbname="citysearch",
            user=args.user,
            password=args.password,
            host=args.host,
            port=args.port)
        cursor = db_conn.cursor()    

    if args.reset_db:
        reset_database(cursor)
    if args.source_tsv:
        load_tsv(args, cursor, args.source_tsv)

def reset_database(cursor):
    print("resetting database")
    cursor.execute(open("reset_database.sql", "r").read())
    print(cursor.connection.notices)
    print("successfully reset")

def load_tsv(args, cursor, source):
    # import csv
    df = pd.read_csv(source,
        sep='\t',
        header=0,
        quotechar='\u00A7',
        encoding='utf-8',
        engine='python')
    print("imported ", source, " with\n", str(df.columns.values), "\nand", str(df.shape), "shape")

    # replace apostrophes with double apostrophes for postgres
    df = df.applymap(lambda x : x.replace("'","''") if type(x) is str else x)
    # escape those nasty random quote characters that will confuse anything that tries to use the data
    df = df.applymap(lambda x : x.replace("\"", "\\\"") if type(x) is str else x)

    # tokenize and strip the alt names
    df['alt_name'] = df['alt_name'].apply(lambda ls: ([tok.strip() for tok in ls.split(',')] if type(ls) is str else ""))

    # join the alt names separately 
    # because otherwise the lambda becomes a mess!
    df['alt_name'] = df['alt_name'].apply(lambda toks: "{" + (",".join(['"{}"'.format(tok) for tok in toks])) + "}")

    # if there's an empty field, we're going to ignore it
    # in real life we wouldn't do this
    # but it'd take way too damn long to figure out what's missing
    # and it probably wouldn't matter anyway
    df = df.fillna(0)

    if (args.dump_hdf):
        df.to_hdf('citysearch.city.h5','root',append=False)

    if (cursor != None):
        print("dumping all rows to database")
        for index, row in df.iterrows():
            command = ("insert into citysearch.city (id, name, ascii, alt_name, lat, long, feat_class, feat_code, country, cc2, admin1, admin2, admin3, admin4, population, elevation, dem, tz, modified_at) values " 
                    + "(" 
                    + ",".join([("'{}'".format(x) if type(x) is str else str(x)) for x in row.values]) 
                    + ");")
            cursor.execute(command)
    print(cursor.connection.notices)


if __name__ == "__main__":
    main()

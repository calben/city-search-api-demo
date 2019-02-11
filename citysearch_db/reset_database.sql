
--- DROP IF SCHEMA EXISTS

drop schema citysearch cascade;

--- CREATE SCHEMA AND IMPORT EXTENSIONS

create schema citysearch;

create extension if not exists postgis;
create extension if not exists postgis_topology;
create extension if not exists postgis_sfcgal;
create extension if not exists fuzzystrmatch;

--- TABLES

create table citysearch.city (
    id                  int primary key unique not null,                         
    name                varchar(200) not null,                                                   
    ascii               varchar(200),                                
    alt_name            text[],                                   
    lat                 double precision not null,                              
    long                double precision not null,                               
    feat_class          text,                                     
    feat_code           text,                                    
    country             text not null,                                  
    cc2                 text,                              
    admin1              varchar(20),                                 
    admin2              varchar(20),                                 
    admin3              varchar(20),                                 
    admin4              varchar(20),                                 
    population          bigint not null check (population >= 0),                                     
    elevation           int,                                    
    dem                 int,                              
    tz                  varchar(40),                             
    modified_at         timestamp not null default now(),    
    position            geometry                                  
);

--- FUNCTIONS

-- this is a little nasty
-- a little complicated for something i'd want in sql
-- but it does the job nicely

-- normally we'd use something like soundex
-- but we also know we're going to potentially get input in a lot of languages
-- perhaps better to run the algorithm with reduced weight for later characters
create or replace function citysearch.city_name_distance(city citysearch.city, input text)
returns int as $$
declare
   distances int[];
   total int;
   input_length int;
begin
   total := array_length(city.alt_name, 1);
   input_length := length(input);
   distances[1] := levenshtein(lower(input), lower(left(city.name, input_length)));
   for i in 2 .. coalesce(total, 0) loop
    distances[i] := levenshtein(input, left(city.alt_name[i], input_length));
   end loop;
return min(x) from unnest(distances) as x;
end;
$$ language plpgsql stable 
returns null on null input;

--- inefficient for many queries because needs to redo makepoint and transform
create or replace function citysearch.city_position_distance(city citysearch.city, lat double precision, long double precision)
returns double precision as $$
  select st_distancesphere(city.position, st_setsrid(st_makepoint(lat, long), 4326))
$$ language sql stable;

create type citysearch.city_distance_result as (city citysearch.city, name_distance int, position_distance double precision);

create or replace function citysearch.all_city_position_distances(geometry)
returns setof citysearch.city_distance_result as $$
  select c::citysearch.city, 0, st_distancesphere(c.position, $1) as record
  from (select * from citysearch.city) as c
$$ language sql stable;


create or replace function citysearch.all_city_name_distances(input text)
returns setof citysearch.city_distance_result as $$
  select c::citysearch.city, citysearch.city_name_distance(c, input), 0::double precision as record
  from (select * from citysearch.city) as c
$$ language sql stable;

# hullo reviewers!

(for everyone else, this codebase was for a company's code challenge and will be taken down shortly)

This is my solution to the citysearch api question.
It features two parts, the database and api components.
Initially I was intending to write a Rust library for all the scoring algorithms that would both be imported by the web server for use and used as a set of stored functions for Postgres.
After that, it'd be pretty straightforward to benchmark them against each other (after Postgres has been "warmed up").
I've been interested in Rust for a while (even started learning it briefly a bit ago but didn't get very far before refocusing on other projects), and I've also been interested in Postgres performance for a "map reduce"-like workflow.

A lot of programmers have commented that Rust is difficult to get used to.
Usually when people say this about a language it's rubbish, but as it turns out for Rust it's really true!
Even having some foundation in working with pointers and smart pointers and some foundation in functional features like closures and currying, Rust has proven to be a confusing beasty even for writing a small application.
For this particular application, I got to tangle with lifetimes in parallel and asynchronous code, and ended up spending a solid hour or so just on figuring out a basic bit of string manipulation.
Most of the code was done Sunday afternoon, and I didn't get as far as I'd have liked.
Here's the stuff that I would have liked to have gotten to:

- Adjust from using `CityRecord::to_cityresult` for getting a `CityResult` with a prepared score value to using a collection of weighted score structs that can be folded to produce a final score, debugged more easily, and easily added to or removed from to change how scoring is done.
- Use the `rayon` library for the parallelised version of the code.
- Finish preparing the library functions as Postgres extensions.  Postgres expects its extensions to be in C.  There is a library that helps set this up for Rust, though, so it would be interesting to see how difficult that is to do.
- Return a `Result<HttpRequest>` instead of a raw `HttpRequest` to improve error handling and reliability in case of a system error and to prevent system panic.
- Prepare more tests.
- Prepare proper benchmarking.
- Fix the `suggestions_postgres`.  This is the function that I prepared first.  As it became clear I wasn't going to have time to implement the database extensions, I stopped developing this to focus on just the in memory solution.
- Rewrite the `db_utils` from Python to Rust.  I intended to go into this project doing everything in Rust, but it became evident that I was going to be low on time.  I've done a looooot of data cleanup in Python, so doing the script to clean up the tsv file and insert everything into the database in Python was very quick to do and may have been long to do in Rust, which doesn't have a library quite like Pandas yet.

Anywhoo, looking forward to talking about this solution.
Please don't hesitate to open an issue or send me a message if something needs clarification before the chat.


Cheers,
Calem

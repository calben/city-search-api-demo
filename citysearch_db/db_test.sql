
--- AFTER DATA IMPORT

update citysearch.city
set position = st_setsrid(st_makepoint(long, lat), 4326);

--- TESTING

select id, name, long, lat, position from citysearch.city limit 10;

select st_distance_sphere(st_setsrid(st_makepoint(-122, 49), 4326), c.position) from
(select id, name, lat, long, position from citysearch.city limit 10) as c;

-- select (city).id, (city).name, (city).long, (city).lat, name_distance, position_distance from citysearch.all_city_position_distances(st_setsrid(st_makepoint(-122, 49), 4326)) order by position_distance limit 10;

-- select (city).id, (city).name, (city).long, (city).lat, name_distance, position_distance from citysearch.all_city_name_distances('Hal') order by name_distance limit 10;


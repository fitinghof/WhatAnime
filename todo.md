
-------------------------------------------------------------------------------------- back-end --------------------------------------------------------------------------------------

 * Spotify groups exists, there for, for database purposes, the same spotify id should be made bindable to multiple artists in some way.

 * Add manual anime bind script 

 * Make user 'profiles' or rather save some info about the user in the database. Could include stuff like binds made, reports made, database mod perhaps to allow free binding and then only allow everyone else to make bind requests.

 * If no anisongs are found (typically due to ranked) in else of if 'certainty == 100.0' make it so that if the db search comese out fine it should count that as a hit and return that.

 * Add artist (CV.|Vo.|CV:[artistname]) unwrapping for anisong search and probably internal also soon

 * Make anisong part of database entries also get updated, this should prove minimal overhead but provide potential extra data to the anime every now and then

 * make artists update their data every now and then

 * Might want to add a last updated for each artist so that perhaps once a week or something I fetch every anime that artist has made and unless it is time for that update I depend 
 fully on the database to choose atleast more_by_artists 

-------------------------------------------------------------------------------------- features --------------------------------------------------------------------------------------

 * Add song report button

 * Add filters for openings / inserts / endings

 * Group stuff from the same anime?

 * Make more by artists include composer and arranger, perhaps in it's own section.

---------------------------------------------------------------------------------------- Bugs ----------------------------------------------------------------------------------------



--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------

 * Not to self: Don't make a database migration in a development branch as that will cause a vesion mismatch
 * If I do, 
 'sqlx migrate info' for local checksum of failed
  UPDATE _sqlx_migrations SET checksum = E'\\xlocal_checksum' WHERE version = failed version
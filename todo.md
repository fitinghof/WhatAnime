 * Group stuff from the same anime?

 * Spotify groups exists, there for, for database purposes, the same spotify id should be made bindable to multiple artists in some way.

 * Make user 'profiles' or rather save some info about the user in the database. Could include stuff like binds made, reports made, database mod perhaps to allow free binding and then only allow everyone else to make bind requests.

 * Make more by artists include composer and arranger, perhaps in it's own section.

 * If no anisongs are found (typically due to ranked) in else of if 'certainty == 100.0' make it so that if the db search comese out fine it should count that as a hit and return that.

 * Rework the raw anisong search

 * Make anisong part of database entries also get updated, this should prove minimal overhead but provide potential extra data to the anime every now and then

 * make artists update their data every now and then

 * Might want to add a last updated for each artist so that perhaps once a week or something I fetch every anime that artist has made and unless it is time for that update I depend fully on the database to choose atleast more_by_artists 

 * Fix so that hit animes will only get updated if there is a need for the update. Not to big of a problem but currently uneccessary database query.

 * When we get a database semi miss and the hit anime, as decided by anisongdb is something with artists not in the spotify song we are listening to all those songs will be counted as new
 and sent to the database for insertion. This is due to the 'full_db_search' function not adding extra anime to more by artists unless it is certain that it got the right anime
 since we then assume that the database finds everything that it should find this leads to the miss and the attempted add of stuff that is already added. One solution to this could be 
 splitting the 'add_or_update' function to separate 'add' and 'update' functions where in the add function we could do a simple precheck fething the ids of animes that are already there. this would be much cheaper than trying to insert, in some cases 300+ animes that are already there.

 * Not to self: Don't make a database migration in a development branch as that will cause a vesion mismatch
 * If I do, 
 'sqlx migrate info' for local checksum of failed
  UPDATE _sqlx_migrations SET checksum = E'\\xlocal_checksum' WHERE version = failed version
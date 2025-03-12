 * Group stuff from the same anime?

 * Make more by artists include composer and arranger, perhaps in it's own section.

 * Make a DNS domain

 * If no anisongs are found (typically due to ranked) in else of if 'certainty == 100.0' make it so that if the db search comese out fine it should count that as a hit and return that.

 * Rework the raw anisong search

 * Fix so that it will only send stuff that really needs to be updated to the update_or_add func

 * Not to self: Don't make a database migration in a development branch as that will cause a vesion mismatch
 * If I do, 
 'sqlx migrate info' for local checksum of failed
  UPDATE _sqlx_migrations SET checksum = E'\\xlocal_checksum' WHERE version = failed version
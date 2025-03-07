 * remove spotify id from animes in next migration -- implemented not pushed

 * Add more anime info

 * Group stuff from the same anime?

 * Make more by artists include composer and arranger, perhaps in it's own section.

 * Make a DNS domain

 * Finish new get anime function

 * Start using branches for new features and stop being an idiot

 * There might be a bug where it does not search by every available artist, example being Take me to the beach with ado and imagine dragons where both Ado and Imagine Dragons would be listed 
 as artists on spotifies side, by this logic the site should show ado's stuff as possible matches but it doesn't.

 * Make a spotify bind a non-neccesity, in other words, add every song found from anisong to the db, even if it does not yet have a bind.
 This would require:
   * Making song_group_id nullable -- implemented not pushed
   * Logic for matching anisong with database song (Should be trivial)
 * This would allow:
   * More advanced song searches
   * Less Anilist API calls, although we might want these for updating the database
   * Less relience on APIs

 * Scrape...

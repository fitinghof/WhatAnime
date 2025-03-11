from anisongdb import (
    AnisongDB_Interface,
    Search_Filter,
    Search_Request,
    Song_Entry,
    Artist_ID_Search_Request,
)
from database import DataBase

sr = Search_Request(
    artist_search_filter=Search_Filter(
        search="Hiroyuki Sawano",
        partial_match=False,
    )
)

if __name__ == "__main__":
    anisong_db = AnisongDB_Interface()
    db = DataBase()
    artist = anisong_db.get_songs(sr)[0].artists[0]
    artist_spotify_id = "0Riv2KnFcLZA3JSVryRg4y"
    inp = input(
        f"Is this the correct artist? {artist.names}\n"
        "Does this link to the artist?\n"
        f"https://open.spotify.com/artist/{artist_spotify_id}\n"
        "If stuff looks good typ 'y'\n"
    )

    if inp == "y" or inp == "Y":
        db.conn.execute(
            """INSERT INTO artists
            (spotify_id, ann_id, names, groups_ids, members)
            VALUES (%s, %s, %s, %s, %s)""",
            (
                artist_spotify_id,
                artist.id,
                artist.names,
                [a.id for a in artist.groups] if artist.groups else None,
                [a.id for a in artist.members] if artist.members else None,
            ),
        )
        db.conn.commit()

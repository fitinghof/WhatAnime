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
        search="Kamisama, Boku wa Kizuite Shimatta",
        partial_match=False,
    )
)

if __name__ == "__main__":
    anisong_db = AnisongDB_Interface()
    db = DataBase()
    artist = anisong_db.get_songs(sr)[0].artists[0]
    print(artist.names)
    artist_spotify_id = "19hnen14uXCUMoBAnTmrCp"
    inp = input(
        f"""Is this the correct artist?{artist.names}
        Does this link to the artist?
        https://open.spotify.com/artist/{artist_spotify_id}
        If stuff looks good typ 'y'
        """
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

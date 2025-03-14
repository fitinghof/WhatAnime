from anisongdb import (
    AnisongDB_Interface,
    Search_Filter,
    Search_Request,
    Song_Entry,
    Artist_ID_Search_Request,
)
from database import DataBase

# Fuuka Izumi, Aoi Koga, Shiori Sugiura
artist_spotify_id = "2uTf3MjHEyidMXyrIh5QzR"
artist_name = "Aoi Koga"
sr = Search_Request(
    artist_search_filter=Search_Filter(
        search=artist_name,
        partial_match=False,
    )
)

if __name__ == "__main__":
    anisong_db = AnisongDB_Interface()
    db = DataBase()
    artist = None
    for possible_artist in anisong_db.get_songs(sr)[0].artists:
        if artist_name in possible_artist.names:
            artist = possible_artist
            break
    if artist is None:
        print("Couldn't find the artist")
        exit()
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

from database import DataBase
from anisongdb import AnisongDB_Interface, Search_Request, Search_Filter


def bind_song() -> bool:
    # return NotImplementedError
    anisong_db = AnisongDB_Interface()
    db = DataBase()
    spotify_id = input("Please provide the spotify id below\n")
    inp = input(
        f"Does this link to the correct song?\nhttps://open.spotify.com/track/{spotify_id}"
    )
    if inp.lower().strip() not in ("y", "yes"):
        return False
    song_name = input("Please write the anime song name\n")
    s = Search_Request(
        song_name_search_filter=Search_Filter(search=song_name, partial_match=False)
    )
    anime_songs = anisong_db.get_songs(s)
    anime_song = None
    for song in anime_songs:
        inp = input(
            f"Is this the correct song?\n{song.songName}\n{[a.names for a in song.artists]}"
        )
        if inp in ("y", "yes"):
            anime_song = song
            break
    if anime_song is None:
        print("Couldn't find the song")
        return False

    # Painfull insert since we transform some strings to i16 on the backend which must be done here to

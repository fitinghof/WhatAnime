from database import DataBase
from anisongdb import AnisongDB_Interface, Search_Request, Search_Filter


def bind_song() -> bool:
    # return NotImplementedError
    anisong_db = AnisongDB_Interface()
    db = DataBase()
    spotify_id: str = input("Please provide the spotify id below\n")
    inp = input(
        f"Does this link to the correct song?\nhttps://open.spotify.com/track/{spotify_id}\n"
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
            f"Is this the correct song?\nSong Title: {song.songName}\nArtists: {[a.names for a in song.artists]}\n"
        )
        if inp in ("y", "yes"):
            anime_song = song
            break
    if anime_song is None:
        print("Couldn't find the song")
        return False

    artist_ids: list[int] = [int(a.id) for a in anime_song.artists]

    cursor = db.conn.cursor()
    cursor.execute(
        "SELECT group_id FROM song_groups WHERE song_title = %s AND artist_ids = %s::integer[]",
        (anime_song.songName, artist_ids),
    )
    group_id = cursor.fetchone()
    if group_id is None:
        cursor.execute(
            "INSERT INTO song_groups (song_title, artist_ids) VALUES (%s, %s) RETURNING group_id",
            (anime_song.songName, artist_ids),
        )
        group_id = cursor.fetchone()

    if group_id is None:
        print("Something went bad")
        return False

    group_id = group_id["group_id"]

    cursor.execute(
        "INSERT INTO song_group_links (spotify_id, group_id) VALUES (%s, %s)",
        (spotify_id, group_id),
    )

    # this part is still needed
    cursor.execute(
        "UPDATE animes SET song_group_id = %s WHERE song_name = %s AND artists_ann_id = %s::integer[] RETURNING (title_eng)",
        (group_id, anime_song.songName, artist_ids),
    )

    print("Song is successfully linked")
    bound_animes = [a["title_eng"] for a in cursor.fetchall()]

    print(f"Binded following animes:\n{bound_animes}")

    db.conn.commit()
    cursor.close()


if __name__ == "__main__":
    bind_song()

# 5P8lyudWE7HQxb4ludLbEm
# Renai Circulation

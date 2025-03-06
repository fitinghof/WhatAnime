import psycopg
from dotenv import load_dotenv
import os
from database import (
    SongGroup,
    DBAnime,
    SongGroupLink,
)
from pydantic import TypeAdapter

from anisongdb import (
    AnisongDB_Interface as AnisongDBI,
    Search_Request,
    Search_Filter,
    Song_Entry,
)


def fetch_animes(database_url: str):
    conn = psycopg.connect(database_url)
    cur = conn.cursor()

    cur.execute(
        "SELECT * FROM animes;",
    )
    result = cur.fetchall()
    columns = [
        desc[0] for desc in cur.description
    ]  # Get column names from the cursor description
    dict_result = [dict(zip(columns, row)) for row in result]

    # Use TypeAdapter to validate the list of dictionaries
    return TypeAdapter(list[DBAnime]).validate_python(dict_result)


def fetch_group(song_title, artist_ids, database_url) -> SongGroup:
    conn = psycopg.connect(database_url)
    cur = conn.cursor()

    cur.execute(
        "SELECT * FROM song_groups WHERE song_title = %s AND artist_ids = %s::integer[]",
        (song_title, artist_ids),
    )
    result = cur.fetchone()

    if result is None:
        return None  # return None if no link exists

    columns = [
        desc[0] for desc in cur.description
    ]  # Get column names from the cursor description
    dict_result = dict(zip(columns, result))  # Map the single row to a dictionary

    # Use TypeAdapter to validate the dictionary
    return TypeAdapter(SongGroup).validate_python(dict_result)


def insert_group(anime: DBAnime, database_url):
    conn = psycopg.connect(database_url)
    cur = conn.cursor()

    cur.execute(
        "INSERT INTO song_groups (song_title, artist_ids) VALUES (%s, %s) ON CONFLICT DO NOTHING RETURNING group_id",
        (anime.song_name, anime.artists_ann_id),
    )

    group_id = cur.fetchone()

    if group_id:
        group_id = group_id[0]
    else:
        group_id = None

    conn.commit()
    cur.close()
    conn.close()

    return group_id


def fetch_link(spotify_song_id: str, database_url):
    conn = psycopg.connect(database_url)
    cur = conn.cursor()

    cur.execute(
        "SELECT * FROM song_group_links WHERE spotify_id = %s",
        (spotify_song_id,),
    )
    result = cur.fetchone()

    if result is None:
        return None  # return None if no link exists

    columns = [
        desc[0] for desc in cur.description
    ]  # Get column names from the cursor description
    dict_result = dict(zip(columns, result))  # Map the single row to a dictionary

    # Use TypeAdapter to validate the dictionary
    return TypeAdapter(SongGroupLink).validate_python(dict_result)


def add_link(spotify_song_id: str, group_id: str, database_url):
    conn = psycopg.connect(database_url)
    cur = conn.cursor()

    print(spotify_song_id)

    cur.execute(
        "INSERT INTO song_group_links (spotify_id, group_id) VALUES (%s, %s) ON CONFLICT DO NOTHING",
        (spotify_song_id, group_id),
    )

    conn.commit()
    cur.close()
    conn.close()


def update_anime(ann_song_id: str, group_id: int, database_url):
    conn = psycopg.connect(database_url)
    cur = conn.cursor()

    cur.execute(
        "UPDATE animes SET song_group_id = %s WHERE ann_song_id = %s",
        (group_id, ann_song_id),
    )

    conn.commit()
    cur.close()
    conn.close()


if __name__ == "__main__":
    load_dotenv()
    database_url = os.getenv("DATABASE_URL")

    db_animes = fetch_animes(database_url)

    current_group_id = 0
    for db_anime in db_animes:
        link = fetch_link(db_anime.spotify_id, database_url)
        group_id: int
        if link is None:
            group = fetch_group(
                db_anime.song_name, db_anime.artists_ann_id, database_url
            )
            if group is None:
                group_id = insert_group(db_anime, database_url)
                add_link(db_anime.spotify_id, group_id, database_url)
            else:
                group_id = group.group_id
                add_link(db_anime.spotify_id, group_id, database_url)
        else:
            group_id = link.group_id
        update_anime(db_anime.ann_song_id, group_id, database_url)

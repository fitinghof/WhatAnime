import psycopg
from psycopg.rows import dict_row
from pydantic import BaseModel
from anisongdb import (
    AnisongDB_Interface,
    Search_Filter,
    Search_Request,
    Song_Entry,
    Artist_ID_Search_Request,
)
from database import DBAnime, DataBase

from datetime import date, datetime


def update_anilist(ann_song_id: int) -> bool:
    db = DataBase()
    db.conn.row_factory = dict_row
    cursor = db.conn.cursor()

    cursor.execute("SELECT * FROM animes WHERE ann_song_id = %s", (ann_song_id,))
    row = cursor.fetchone()
    if row is None:
        print(f"No anime found with ann_song_id = {ann_song_id}")
        return False
    db_anime = DBAnime(**row)
    inp = input(
        f"Is this the anime you want to update the anilist for?\n\t{db_anime.title_eng}\n"
    )

    if not inp.lower().strip() in ("y", "yes"):
        print("Did nothing")
        return False

    anilist_id = db_anime.anilist_id

    inp = input("Do you want to provide a new anilist id for this anime?\n")
    if inp.lower().strip() in ("y", "yes"):
        try:
            anilist_id = int(input("Write the new anilist id\n").strip())
        except:
            print("Could not make input into a number")
            return False

    if anilist_id is None:
        if db_anime.anilist_id is None:
            print("No anilist id to update found, try providing one next time")
            return False

    inp = input(
        f"If this is the correct anilist please type (y)es \n\thttps://anilist.co/anime/{anilist_id}\n"
    )

    if not inp.lower().strip() in ("y", "yes"):
        print("Did nothing")
        return False

    cursor.execute(
        "UPDATE animes SET anilist_id = %s, last_updated = '2000-03-14 13:58:08.675223+01' WHERE ann_song_id = %s",
        (anilist_id, ann_song_id),
    )

    db.conn.commit()
    cursor.close()

    return True


if __name__ == "__main__":
    update_anilist()

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

from remove_song_link import remove_song_link

from update_anilist import update_anilist

from utility import clear_screen


class Report(BaseModel):
    report_id: int
    spotify_id: str
    ann_song_id: int
    reason: str
    user_name: str
    user_mail: str
    date_added: datetime

    def __repr__(self):
        return f"Report(id={self.id}, spotify_song_id={self.spotify_id}, ann_song_id={self.ann_song_id}, reason={self.reason}, user_name={self.user_name}, user_email={self.user_mail}, timestamp={self.date_added})"


def parse_repports():
    db = DataBase()
    db.conn.row_factory = dict_row
    cursor = db.conn.cursor()

    cursor.execute("SELECT * FROM reports")

    reports = [Report(**row) for row in cursor.fetchall()]

    if len(reports) == 0:
        print("No Reports! it is a happy day :)")
        return

    for report in reports:
        clear_screen()

        cursor.execute(
            "SELECT * FROM animes WHERE ann_song_id = %s", (report.ann_song_id,)
        )
        db_anime = DBAnime(**cursor.fetchone())

        print(db_anime.model_dump_json(indent=4))
        print()

        print(report.user_name)
        print(report.user_mail)
        print(report.date_added)
        print(report.reason)
        print(f"spotify link: https://open.spotify.com/track/{report.spotify_id}\n")

        inp = input(
            "What would you like to do?\n"
            "(s)kip, (w)ipe song link, (r)emove report, (u)pdate anilist\n"
        )
        match inp.lower().strip():
            case "s":
                continue
            case "w":
                if remove_song_link(report.spotify_id, db=db):
                    cursor.execute(
                        "DELETE FROM reports WHERE report_id = %s", (report.report_id,)
                    )
            case "r":
                cursor.execute(
                    "DELETE FROM reports WHERE report_id = %s", (report.report_id,)
                )
                print(f"Deleted report")
            case "u":
                if update_anilist(report.ann_song_id):
                    cursor.execute(
                        "DELETE FROM reports WHERE report_id = %s", (report.report_id,)
                    )

            case _:
                print("invalid input, did nothing")
    db.conn.commit()
    cursor.close()

    print("All reports parsed : )")


if __name__ == "__main__":
    parse_repports()

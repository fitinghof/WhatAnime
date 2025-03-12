from anisongdb import (
    AnisongDB_Interface,
    Search_Filter,
    Search_Request,
    Song_Entry,
    Artist_ID_Search_Request,
)
from database import DataBase


if __name__ == "__main__":
    db = DataBase()
    spotify_id = "0Z51sIImtvHFIVomgeS1R7"

    cursor = db.conn.cursor()
    confirm = input(f"correct link? : https://open.spotify.com/track/{spotify_id}\n")
    if confirm == "y" or confirm == "Y":
        cursor.execute(
            "DELETE FROM song_group_links WHERE spotify_id = %s RETURNING group_id",
            (spotify_id,),
        )

        group_ids = [row[0] for row in cursor.fetchall()]

        if group_ids:
            cursor.execute(
                "DELETE FROM animes WHERE song_group_id = ANY(%s)", (group_ids,)
            )
        else:
            print("Couldn't find the song")

        db.conn.commit()
        cursor.close()
        print("Done :)")
    else:
        print("Nothing done")

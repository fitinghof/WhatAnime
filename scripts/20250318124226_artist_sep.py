from database import *

from psycopg.rows import dict_row

if __name__ == "__main__":
    db = DataBase()
    db.conn.row_factory = dict_row
    cursor = db.conn.cursor()
    cursor.execute("SELECT * FROM artists")
    unparsed = cursor.fetchall()
    if unparsed is None:
        print("No artist found")
        exit()
    artists = [DBArtist(**row) for row in unparsed]

    new_artists = [(a.ann_id, a.names, a.groups_ids, a.members) for a in artists]
    links = [(a.ann_id, a.spotify_id) for a in artists]

    cursor.executemany(
        "INSERT INTO new_artists (ann_id, names, groups_ids, members) VALUES (%s, %s, %s, %s) ON CONFLICT DO NOTHING",
        new_artists,
    )

    cursor.executemany(
        "INSERT INTO artist_links (ann_id, spotify_id) VALUES (%s, %s) ON CONFLICT DO NOTHING",
        links,
    )

    db.conn.commit()
    cursor.close()

from pydantic import BaseModel
from typing import Optional
import psycopg
import os
from dotenv import load_dotenv


class DBAnime(BaseModel):
    ann_id: int
    title_eng: str
    title_jpn: str
    index_type: int
    index_number: int
    anime_type: int
    episodes: Optional[int] = None
    mean_score: Optional[int] = None
    banner_image: Optional[str] = None
    cover_image_color: Optional[str] = None
    cover_image_medium: Optional[str] = None
    cover_image_large: Optional[str] = None
    cover_image_extra_large: Optional[str] = None
    media_format: Optional[int] = None
    genres: Optional[list[str]] = None
    source: Optional[str] = None
    studio_ids: Optional[list[int]] = None
    studio_names: Optional[list[str]] = None
    studio_urls: Optional[list[str]] = None
    tag_ids: Optional[list[int]] = None
    tag_names: Optional[list[str]] = None
    trailer_id: Optional[str] = None
    trailer_site: Optional[str] = None
    thumbnail: Optional[str] = None
    release_year: Optional[int] = None
    release_season: Optional[int] = None
    spotify_id: str
    ann_song_id: int
    song_name: str
    spotify_artist_ids: list[str]
    artist_names: list[str]
    artists_ann_id: list[int]
    composers_ann_id: list[int]
    arrangers_ann_id: list[int]
    track_index_type: int
    track_index_number: int
    mal_id: Optional[int] = None
    anilist_id: Optional[int] = None
    anidb_id: Optional[int] = None
    kitsu_id: Optional[int] = None


class SongGroup(BaseModel):
    group_id: int
    song_title: str
    artist_ids: list[int]


class SongGroupLink(BaseModel):
    spotify_id: str
    group_id: int


class DataBase:
    def __init__(self):
        load_dotenv()
        database_url = os.getenv("DATABASE_URL")
        self.conn = psycopg.connect(database_url)

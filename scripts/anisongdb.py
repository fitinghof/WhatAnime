import requests
from typing import List, Optional
from pydantic import BaseModel, Field, TypeAdapter


class Search_Filter(BaseModel):
    search: str
    partial_match: Optional[bool] = True

    # How much I decompose the group to search for other songs
    # ie. 1: Artists one by one 2: At least two member from the group, etc...
    group_granularity: Optional[int] = Field(0, ge=0)
    # Once I've confirmed group_granularity requirement is met
    # How much other artists that are not from the og group do I accept
    max_other_artist: Optional[int] = Field(99, ge=0)

    # for composer search
    arrangement: Optional[bool] = True


class Search_Request(BaseModel):
    anime_search_filter: Optional[Search_Filter] = None
    song_name_search_filter: Optional[Search_Filter] = None
    artist_search_filter: Optional[Search_Filter] = None
    composer_search_filter: Optional[Search_Filter] = None

    and_logic: Optional[bool] = True

    ignore_duplicate: Optional[bool] = False

    opening_filter: Optional[bool] = True
    ending_filter: Optional[bool] = True
    insert_filter: Optional[bool] = True

    normal_broadcast: Optional[bool] = True
    dub: Optional[bool] = True
    rebroadcast: Optional[bool] = True

    standard: Optional[bool] = True
    instrumental: Optional[bool] = True
    chanting: Optional[bool] = True
    character: Optional[bool] = True


class Artist_ID_Search_Request(BaseModel):
    artist_ids: List[int] = []
    group_granularity: Optional[int] = Field(99, ge=0)
    max_other_artist: Optional[int] = Field(0, ge=0)
    ignore_duplicate: Optional[bool] = False

    opening_filter: Optional[bool] = True
    ending_filter: Optional[bool] = True
    insert_filter: Optional[bool] = True

    normal_broadcast: Optional[bool] = True
    dub: Optional[bool] = True
    rebroadcast: Optional[bool] = True

    standard: Optional[bool] = True
    instrumental: Optional[bool] = True
    chanting: Optional[bool] = True
    character: Optional[bool] = True


class Composer_ID_Search_Request(BaseModel):
    composer_ids: List[int] = []
    arrangement: Optional[bool] = True
    ignore_duplicate: Optional[bool] = False

    opening_filter: Optional[bool] = True
    ending_filter: Optional[bool] = True
    insert_filter: Optional[bool] = True

    normal_broadcast: Optional[bool] = True
    dub: Optional[bool] = True
    rebroadcast: Optional[bool] = True

    standard: Optional[bool] = True
    instrumental: Optional[bool] = True
    chanting: Optional[bool] = True
    character: Optional[bool] = True


class annId_Search_Request(BaseModel):
    annId: int
    ignore_duplicate: Optional[bool] = False

    opening_filter: Optional[bool] = True
    ending_filter: Optional[bool] = True
    insert_filter: Optional[bool] = True

    normal_broadcast: Optional[bool] = True
    dub: Optional[bool] = True
    rebroadcast: Optional[bool] = True

    standard: Optional[bool] = True
    instrumental: Optional[bool] = True
    chanting: Optional[bool] = True
    character: Optional[bool] = True


class malIds_Search_Request(BaseModel):
    malIds: List[int] = []
    ignore_duplicate: Optional[bool] = False

    opening_filter: Optional[bool] = True
    ending_filter: Optional[bool] = True
    insert_filter: Optional[bool] = True

    normal_broadcast: Optional[bool] = True
    dub: Optional[bool] = True
    rebroadcast: Optional[bool] = True

    standard: Optional[bool] = True
    instrumental: Optional[bool] = True
    chanting: Optional[bool] = True
    character: Optional[bool] = True


class artist(BaseModel):
    id: int
    names: List[str]
    line_up_id: Optional[int]
    groups: Optional[List["artist"]]
    members: Optional[List["artist"]]


artist.model_rebuild()


class Anime_List_Links(BaseModel):
    myanimelist: Optional[int]
    anidb: Optional[int]
    anilist: Optional[int]
    kitsu: Optional[int]


class Song_Entry(BaseModel):
    annId: int
    annSongId: int
    animeENName: str
    animeJPName: str
    animeAltName: Optional[List[str]]
    animeVintage: Optional[str]
    linked_ids: Anime_List_Links
    animeType: Optional[str]
    animeCategory: Optional[str]
    songType: str
    songName: str
    songArtist: str
    songComposer: str
    songArranger: str
    songDifficulty: Optional[float]
    songCategory: Optional[str]
    songLength: Optional[float]
    isDub: Optional[bool]
    isRebroadcast: Optional[bool]
    HQ: Optional[str]
    MQ: Optional[str]
    audio: Optional[str]
    artists: List[artist]
    composers: List[artist]
    arrangers: List[artist]

    def __hash__(self):
        return hash((self.annId, self.annSongId, self.songType))

    def __eq__(self, other):
        if isinstance(other, Song_Entry):
            return (
                self.annId == other.annId
                and self.annSongId == other.annSongId
                and self.songType == other.songType
            )
        return False


# Make the request for all songs from the specified artist


class AnisongDB_Interface:
    _site: str

    def __init__(self, site="https://anisongdb.com/api"):
        self._site = site

    def get_songs(self, search: Search_Request) -> List[Song_Entry]:
        respons: requests.models.Response = requests.post(
            self._site + "/search_request", json=search.model_dump()
        )
        return list(set(TypeAdapter(List[Song_Entry]).validate_python(respons.json())))

    def get_exact_song(self, songName, artistIDs: List) -> List[Song_Entry]:
        songlist = self.get_songs_artists(artistIDs, True)

        filteredSonglist = list(filter(lambda a: a.songName == songName, songlist))

        return filteredSonglist

    def get_songs_artists(
        self, artistIDs, everyArtist: bool = False
    ) -> List[Song_Entry]:
        songlist: List = []
        if everyArtist:
            search = Artist_ID_Search_Request(
                artist_ids=artistIDs,
                max_other_artist=0,
            )
            respons: requests.models.Response = requests.post(
                self._site + "/artist_ids_request", json=search.model_dump()
            )
            print(f"Response Status Code: {respons.status_code}")
            print(f"Response Text: {respons.text}")
            songlist = TypeAdapter(List[Song_Entry]).validate_python(respons.json())
        else:
            for artistID in artistIDs:
                search = Artist_ID_Search_Request(
                    artist_ids=[artistID],
                    max_other_artist=99,
                )
                respons: requests.models.Response = requests.post(
                    self._site + "/artist_ids_request", json=search.model_dump()
                )
                print(f"Response Status Code: {respons.status_code}")
                print(f"Response Text: {respons.text}")
                songlist.extend(
                    TypeAdapter(List[Song_Entry]).validate_python(respons.json())
                )
        return list(set(songlist))

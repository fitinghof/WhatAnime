import React from "react";

export interface AnimeIndex {
  Season?: number,
  Movie?: number,
  ONA?: number,
  OVA?: number,
  TVSpecial?: number,
}

function parseAnimeIndex(animeIndex: AnimeIndex): string {
  if (animeIndex.Season !== undefined) return `Season ${animeIndex.Season ? animeIndex.Season : 1}`;
  if (animeIndex.Movie !== undefined) return `Movie ${animeIndex.Movie ? animeIndex.Movie : 1}`;
  if (animeIndex.ONA !== undefined) return `ONA ${animeIndex.ONA ? animeIndex.ONA : 1}`;
  if (animeIndex.OVA !== undefined) return `OVA ${animeIndex.OVA ? animeIndex.OVA : 1}`;
  if (animeIndex.TVSpecial !== undefined) return `OVA ${animeIndex.TVSpecial ? animeIndex.TVSpecial : 1}`;
  console.log(animeIndex);
  return "wacky season"
}

export interface AnimeTrackIndex {
  Opening?: number;
  Insert?: number;
  Ending?: number;
}

function parseTrackIndex(track: AnimeTrackIndex): string {
  if (track === undefined) return "";
  if (track.Opening !== undefined) return `Opening ${track.Opening ?? ""}`;
  if (track.Insert !== undefined) return `Insert Song`;
  if (track.Ending !== undefined) return `Ending ${track.Ending ?? ""}`;
  return "";
}

export enum AnimeType {
  TV = "TV",
  Movie = "Movie",
  OVA = "OVA",
  ONA = "ONA",
}

export interface LinkedIds {
  myanimelist?: number;
  anidb?: number;
  anilist?: number;
  kitsu?: number;
}

export interface AnimeInfo {
  title: string;
  title_japanese: string;
  anime_index: AnimeIndex;
  track_index: AnimeTrackIndex;
  anime_type: AnimeType;
  image_url?: string;
  linked_ids: LinkedIds;

  song_name: string,
  artist_ids: Array<number>,
}

interface AnimeEntryProps {
  anime: AnimeInfo;
  show_confirm_button: boolean,
  spotify_song_id: string,
  show_song_title: boolean,
  after_anime_bind: () => void;
}

const AnimeEntry: React.FC<AnimeEntryProps> = ({ anime, show_confirm_button, spotify_song_id, show_song_title, after_anime_bind }) => {

  const handleConfirmClick = () => {
    const params = {
      song_name: anime.song_name,
      artist_ids: anime.artist_ids,
      spotify_id: spotify_song_id,
    };
    fetch("/api/confirm_anime", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(params)
    })
    .then(response => response.text())
    .then(data => {
      console.log(data);
      after_anime_bind();
    })
  };

  let animeSongNumber = parseTrackIndex(anime.track_index);
  let animeIndex = parseAnimeIndex(anime.anime_index);
  return (
    <div className="anime-item">
      <img
        src={anime.image_url ?? "/Amq_icon_green.png"}
        alt="Anime art"
        className="anime-art"
      />
      <div className="anime-info">
        <div className="anime-title">
          {anime.title || "Unknown Anime"}
        </div>
        {show_song_title &&
          <div className="anime-song-title">
            {`Song Title: ${anime.song_name}`}
          </div>
        }
        {
          <div className="anime-season">
            {`${animeIndex}`}
          </div>
        }
        <div className="anime-opening">
          {animeSongNumber}
        </div>
        <div className="anime-type">
          {`Type: ${anime.anime_type || "Unknown"}`}
        </div>
        {anime.linked_ids && (
          <div className="anime-links">
            {anime.linked_ids.myanimelist && (
              <a
                href={`https://myanimelist.net/anime/${anime.linked_ids.myanimelist}`}
                target="_blank"
                rel="noopener noreferrer"
              >
                MAL
              </a>
            )}
            {anime.linked_ids.anilist && (
              <a
                href={`https://anilist.co/anime/${anime.linked_ids.anilist}`}
                target="_blank"
                rel="noopener noreferrer"
              >
                Anilist
              </a>
            )}
            {anime.linked_ids.anidb && (
              <a
                href={`https://anidb.net/anime/${anime.linked_ids.anidb}`}
                target="_blank"
                rel="noopener noreferrer"
              >
                AniDB
              </a>
            )}
            {anime.linked_ids.kitsu && (
              <a
                href={`https://kitsu.io/anime/${anime.linked_ids.kitsu}`}
                target="_blank"
                rel="noopener noreferrer"
              >
                Kitsu
              </a>
            )}
          </div>
        )}
      </div>

      {show_confirm_button && (
        <button className="anime-button" onClick={handleConfirmClick}>
          <p>
            Confirm<br></br> Anime
          </p>
        </button>
      )}


    </div>
  );
};

export default AnimeEntry;
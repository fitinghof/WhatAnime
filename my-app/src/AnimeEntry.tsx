import React from "react";

export interface AnimeIndex {
  Season: number | null;
  Movie: number | null;
  ONA: number | null;
}

export interface AnimeTrackIndex {
  Opening?: number;
  Insert?: number;
  Ending?: number;
}

function parseTrackIndex(track: AnimeTrackIndex): string {
  if (!track) return "";
  if (track.Opening !== undefined) return `Opening ${track.Opening}`;
  if (track.Insert !== undefined) return `Insert Song`;
  if (track.Ending !== undefined) return `Ending ${track.Ending}`;
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
  image_url: string;
  linked_ids: LinkedIds;
}

interface AnimeEntryProps {
  anime: AnimeInfo;
}

const AnimeEntry: React.FC<AnimeEntryProps> = ({ anime }) => {
  let animeSongNumber = parseTrackIndex(anime.track_index);
  return (
    <div className="anime-item">
      <img src={anime.image_url} alt={`${anime.title} cover`} className="anime-art" />
      <div className="anime-info">
        <div className="anime-title">
          {anime.title || "Unknown Anime"}
        </div>
        <div className="anime-season">
          {`Season ${anime.anime_index ? anime.anime_index.Season : "Unknown"}`}
        </div>
        <div className="anime-opening">
          {anime.track_index?.Opening !== undefined
            ? animeSongNumber
            : ""}
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
    </div>
  );
};

export default AnimeEntry;
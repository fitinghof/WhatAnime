import React, { useEffect, useState } from "react";
import ReportButton from "./report_window";
import './AnimeEntry.css'
export interface AnimeIndex {
  Season?: number,
  Movie?: number,
  ONA?: number,
  OVA?: number,
  TVSpecial?: number,
  Special?: number,
  MusicVideo?: number,
}

function parseAnimeIndex(animeIndex: AnimeIndex): string {
  if (animeIndex.Season !== undefined) return `Season ${animeIndex.Season ? animeIndex.Season : 1}`;
  if (animeIndex.Movie !== undefined) return `Movie ${animeIndex.Movie ? animeIndex.Movie : 1}`;
  if (animeIndex.ONA !== undefined) return `ONA ${animeIndex.ONA ? animeIndex.ONA : 1}`;
  if (animeIndex.OVA !== undefined) return `OVA ${animeIndex.OVA ? animeIndex.OVA : 1}`;
  if (animeIndex.TVSpecial !== undefined) return `TV Special ${animeIndex.TVSpecial ? animeIndex.TVSpecial : 1}`;
  if (animeIndex.Special !== undefined) return `Special ${animeIndex.Special ? animeIndex.Special : 1}`;
  if (animeIndex.MusicVideo !== undefined) return `Music Video ${animeIndex.MusicVideo ? animeIndex.MusicVideo : 1}`;
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
  banner_url?: string;
  linked_ids: LinkedIds;
  score?: number;

  ann_song_id: number;
  song_name: string,
  artist_ids: Array<number>,
  artist_names: Array<String>,
}

interface AnimeEntryProps {
  anime: AnimeInfo;
  show_confirm_button: boolean,
  spotify_song_id: string,
  after_anime_bind: () => void;
}

function linked_ids(anime_ids: LinkedIds) {
  if (anime_ids === undefined) return null;
  return (
    <div className="anime-links">
      {anime_ids.myanimelist && (
        <a
          href={`https://myanimelist.net/anime/${anime_ids.myanimelist}`}
          target="_blank"
          rel="noopener noreferrer"
        >
          MAL
        </a>
      )}
      {anime_ids.anilist && (
        <a
          href={`https://anilist.co/anime/${anime_ids.anilist}`}
          target="_blank"
          rel="noopener noreferrer"
        >
          Anilist
        </a>
      )}
      {anime_ids.anidb && (
        <a
          href={`https://anidb.net/anime/${anime_ids.anidb}`}
          target="_blank"
          rel="noopener noreferrer"
        >
          AniDB
        </a>
      )}
      {anime_ids.kitsu && (
        <a
          href={`https://kitsu.io/anime/${anime_ids.kitsu}`}
          target="_blank"
          rel="noopener noreferrer"
        >
          Kitsu
        </a>
      )}
    </div>
  );
}

const AnimeEntry: React.FC<AnimeEntryProps> = ({ anime, show_confirm_button, spotify_song_id, after_anime_bind }) => {
  const [showMoreInfo, setShowMoreInfo] = useState(false);

  useEffect(() => {
    setShowMoreInfo(false);
  }, [anime]);

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
    <div className="anime-item" style={{ backgroundImage: `linear-gradient(rgba(0, 0, 0, 0.5), rgba(0, 0, 0, 0.5)), url(${anime.banner_url ?? "/amq_icon_green.svg"})` }}>
      <div className="left-info-container">
        <img
          src={anime.image_url ?? "/amq_icon_green.svg"}
          alt="Anime art"
          className="anime-art"
          onError={(e) => {
            e.currentTarget.src = "/amq_icon_green.svg"; // Fallback to SVG
          }}
        />
        {showMoreInfo && (
          <div className="report-button-container">
            <ReportButton ann_song_id={anime.ann_song_id} spotify_song_id={spotify_song_id} />
          </div>
        )}
      </div>
      <div className="anime-info">
        <div className="anime-title">
          {anime.title || "Unknown Anime"}
        </div>

        {showMoreInfo &&
          <div className="extra-info">
            <div className="anime-song-title">
              {`Song Title: ${anime.song_name}`}
            </div>

            <div className="anime-artists-names">
              {`Artists: ${anime.artist_names.join(", ")}`}
            </div>

            <div className="anime-season">
              {`${animeIndex}`}
            </div>

            <div className="anime-opening">
              {animeSongNumber}
            </div>

            <div className="anime-type">
              {`Type: ${anime.anime_type || "Unknown"}`}
            </div>
            {linked_ids(anime.linked_ids)}
          </div>
        }

        {/* Toggle Button */}
        <button className="toggle-extra-info-button" onClick={() => setShowMoreInfo(!showMoreInfo)}>
          {showMoreInfo ? "Hide Info" : "Show More Info"}
        </button>
      </div>
      <div className="right-info-container">
        <div className="anime-score">
          <div className="score-text">
            {
              anime.score ?? ""
            }
          </div>
        </div>
        {show_confirm_button && showMoreInfo && (
          <button className="bind-anime-button" onClick={handleConfirmClick}>
            <p>
              Confirm<br></br> Anime
            </p>
          </button>
        )}
      </div>
    </div>
  );
};

export default AnimeEntry;
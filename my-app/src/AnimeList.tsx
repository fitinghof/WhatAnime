import React from "react";
import AnimeEntry, { AnimeInfo } from "./AnimeEntry";

interface AnimeListProps {
  animes: AnimeInfo[];
}

const AnimeList: React.FC<AnimeListProps> = ({ animes }) => {
  return (
    <div className="anime-list" id="animes">
      {animes.map((anime, index) => (
        <AnimeEntry key={index} anime={anime} />
      ))}
    </div>
  );
};

export default AnimeList;
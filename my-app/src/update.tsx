import { useEffect, useState } from "react";
import AnimeEntry, { AnimeInfo } from "./AnimeEntry"; // use AnimeInfo and AnimeEntry from AnimeEntry.tsx

const Update = () => {
    const [songInfo, setSongInfo] = useState<{
        title: string;
        artists: string[];
        album_picture_url: string;
    } | null>(null);
    const [animeList, setAnimeList] = useState<AnimeInfo[]>([]);
    const [animeList2, setAnimeList2] = useState<AnimeInfo[]>([]);
    const [separator1, setSeparator1] = useState<string>("");
    const [separator2, setSeparator2] = useState<string>("");

    useEffect(() => {
        console.log("Update component mounted");
        const fetchUpdate = (refresh: boolean = false) => {
            const fetch_address = `/api/update${refresh ? "?refresh=true": ""}`
            fetch(fetch_address, { credentials: "include" })
                .then((response) => response.json())
                .then((data) => {
                    console.log(data)
                    if (data.NewSong) {
                        if (data.NewSong.Miss) {
                            setSongInfo({
                                title: data.NewSong.Miss.song_info.title,
                                artists: data.NewSong.Miss.song_info.artists,
                                album_picture_url: data.NewSong.Miss.song_info.album_picture_url,
                            });
                            const animes = data.NewSong.Miss.possible_anime
                            setAnimeList2([]);
                            setAnimeList(animes);
                            setSeparator1(animes.length > 0 ? "Possible matches" : "No matches")
                            setSeparator2("");

                        } else if (data.NewSong.Hit) {
                            const hit = data.NewSong.Hit;
                            console.log(hit.anime_info);
                            setSongInfo({
                                title: hit.song_info.title,
                                artists: hit.song_info.artists,
                                album_picture_url: hit.song_info.album_picture_url,
                            });
                            setSeparator1(`${hit.certainty}% Match`);
                            setAnimeList(hit.anime_info);

                            setSeparator2("More by this artist");
                            setAnimeList2(hit.more_with_artist);
                        }
                    } else if (data === "NotPlaying") {
                        setSongInfo({
                            title: "Not playing anything",
                            artists: [],
                            album_picture_url: "/Amq_icon_green.png",
                        });
                        setAnimeList([]);
                        setAnimeList2([]);
                        setSeparator2("");
                        setSeparator1("");
                    } else if (data === "LoginRequired") {
                        window.location.href = "/api/login";
                    } else if (data == "NoUpdates"){
                    }
                })
            .catch((err) => console.error(err));
        };
        // Run immediately, then every 5 seconds (5000ms)
        fetchUpdate(true);
        const intervalId = setInterval(fetchUpdate, 5000);

        return () => clearInterval(intervalId);

    }, []);

    return (
        <div>
            {/* Adjusted markup to match your CSS */}
            <div className="now-playing-container">
                <div className="now-playing">
                    <img
                        className="album-art"
                        src={songInfo ? songInfo.album_picture_url : "/static/slime.png"}
                        alt="Album cover"
                    />
                    <div className="song-info">
                        <h1 className="song-title">
                            {songInfo ? songInfo.title : "No song info"}
                        </h1>
                        <p className="artist-name">
                            {songInfo ? songInfo.artists.join(", ") : ""}
                        </p>
                    </div>
                </div>
            </div>

            {separator1 && (
                <div className="separator" id="matches">
                    {separator1}
                </div>
            )}
            <div className="anime-list" id="animes">
                {animeList.map((anime, index) => (
                    <AnimeEntry key={index} anime={anime} />
                ))}
            </div>

            {separator2 && (
                <div className="separator" id="matches2">
                    {separator2}
                </div>
            )}

            <div className="anime-list" id="animes2">
                {animeList2.map((anime, index) => (
                    <AnimeEntry key={index} anime={anime} />
                ))}
            </div>
        </div>
    );

};

export default Update;
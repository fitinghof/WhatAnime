import React from "react";
import { useState } from "react";

interface ReportInfo {
    ann_song_id: number,
    spotify_song_id: string,
}

const ReportButton: React.FC<ReportInfo> = ({ ann_song_id, spotify_song_id }) => {
    const [isOpen, setIsOpen] = useState(false); // Controls popup visibility
    const [reason, setReason] = useState(""); // Stores the report reason

    const handleSubmit = () => {
        const params = {
            spotify_id: spotify_song_id,
            ann_song_id: ann_song_id,
            reason: reason,
        };
        fetch("/api/report", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(params)
        })
            .then(response => response.text())
            .then(data => {
                console.log(data);
            })
        console.log("Report Submitted:", reason, ann_song_id, spotify_song_id);
        setIsOpen(false); // Close the popup after submitting
        setReason(""); // Reset the input field
    };

    return (
        <div>
            {/* Report Button */}
            <button
                onClick={() => setIsOpen(true)}
                className="report-button"
            >
                Report
            </button>

            {/* Popup */}
            {isOpen && (
                <div className="report-window-overlay">
                    <div className="popup-container">
                        <h2 className="report-header">Report Issue</h2>
                        <textarea
                            value={reason}
                            onChange={(e) => setReason(e.target.value)}
                            placeholder="What is the reason for the report?"
                            className="report-textarea"
                        />
                        <div className="popup-buttons">
                            <button
                                onClick={() => setIsOpen(false)}
                                className="popup-cancel"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleSubmit}
                                className="popup-submit"
                            >
                                Submit
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};

export default ReportButton;
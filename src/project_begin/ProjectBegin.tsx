import { useState, useEffect } from "react";
import { dialog } from "@tauri-apps/api"; // Import Tauri's dialog API
import React from "react";
import { useNavigate } from "react-router-dom";
import Statics from "../Statics";

function ProjectBegin() {
    const navigate = useNavigate();

    const [projectAccessType, setProjectAccessType] = useState<string>(() => {
        return localStorage.getItem(Statics.PROJECT_ACCESS_TYPE) || "";
    });
    const [projectPath, setProjectPath] = useState<string>(() => {
        return localStorage.getItem(Statics.PROJECT_PATH) || "";
    });

    useEffect(() => {
        localStorage.setItem(Statics.PROJECT_ACCESS_TYPE, projectAccessType);
    }, [projectAccessType]);

    useEffect(() => {
        localStorage.setItem(Statics.PROJECT_PATH, projectPath);
    }, [projectPath]);

    const handleProjectAccessTypeChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
        setProjectAccessType(event.target.value);
    };

    const handleFolderSelection = async () => {
        try {
            const selectedPath = await dialog.open({
                directory: true,
                multiple: false,
            });

            if (selectedPath && typeof selectedPath === "string") {
                setProjectPath(selectedPath);
            }
        } catch (error) {
            console.error("Failed to select folder:", error);
        }
    };

    return (
        <div className="project-begin-form">
            <h1>Select Project</h1>

            <div className="container">
                <div className="row">
                    <label htmlFor="projectAccessType">Project Type:</label>
                    <select
                        id="projectAccessType"
                        value={projectAccessType}
                        onChange={handleProjectAccessTypeChange}
                        className="project-type-select"
                    >
                        <option value="">Select project type</option>
                        <option value="git-source">Git Source</option>
                        <option value="local-project">Local Project</option>
                    </select>
                </div>

                <div className="row">
                    <label htmlFor="projectPath">Project Path:</label>
                    <div className="folder-selection">
                        <button
                            onClick={handleFolderSelection}
                            className="folder-select-button"
                        >
                            {projectPath ? `${projectPath}` : "Select folder"}
                        </button>
                    </div>
                </div>

                <div style={{ height: "24px" }}></div>
                <button
                    onClick={() => navigate('/graph')}
                    className=""
                >
                    Begin
                </button>
            </div>
        </div>
    );
}

export default ProjectBegin;

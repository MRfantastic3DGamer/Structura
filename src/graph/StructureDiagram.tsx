import { useEffect, useState } from "react";
import { Command } from "@tauri-apps/api/shell";
import Statics from "../Statics";
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

function StructureDiagram() {
    const projectPath = localStorage.getItem(Statics.PROJECT_PATH);
    const [D, setD] = useState("");
    const [progress, setProgress] = useState(0); // State to track progress

    useEffect(() => {
        // Listen to the progress event emitted from the backend
        const unlisten = listen('progress', (event) => {
            // event.payload will be the progress value emitted from the backend
            setProgress(event.payload as number);
        });

        // Clean up the event listener on component unmount
        return () => {
            unlisten.then((f) => f());
        };
    }, []);

    const generateTags = async () => {
        if (projectPath) {
            try {
                // Create a new Command instance
                const command = new Command("ctags", ["-R", "--recurse=yes", "-f tags", projectPath]);
                // Execute the command and capture the output
                const output = await command.execute();

                if (output.code === 0) {
                    setD("Tags file generated successfully!");

                    // Use the imported `invoke` function
                    await invoke('request_project_structure', { tagsPath: "tags" });
                } else {
                    setD(`Failed to generate tags. Error: ${output.stderr}`);
                }
            } catch (error) {
                setD(`Failed to execute ctags command: ${error}`);
            }
        }
    };

    return (
        <div>
            <h1>{projectPath}</h1>
            <button onClick={generateTags}>Generate tags</button>
            <h1>{D}</h1>

            {/* Display progress */}
            <p>Progress: {progress}%</p>
        </div>
    );
}
export default StructureDiagram;
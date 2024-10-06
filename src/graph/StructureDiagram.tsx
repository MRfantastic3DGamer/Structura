import { useEffect, useState } from "react";
import { Command } from "@tauri-apps/api/shell";
import Statics from "../Statics";

function StructureDiagram() {
    const projectPath = localStorage.getItem(Statics.PROJECT_PATH);
    const [D, setD] = useState("")

    const generateTags = async () => {
        if (projectPath) {
            try {
                // Create a new Command instance
                const command = new Command("ctags", ["-R", "-f", "./data/tags", projectPath]);

                // Execute the command and capture the output
                const output = await command.execute();

                if (output.code === 0) {
                    setD("Tags file generated successfully!");
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
        </div>
    );
}

export default StructureDiagram;

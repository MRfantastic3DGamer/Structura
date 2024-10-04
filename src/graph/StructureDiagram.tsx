import { useEffect } from "react";
import { Command } from "@tauri-apps/api/shell";
import Statics from "../Statics";

function StructureDiagram() {
    const projectPath = localStorage.getItem(Statics.PROJECT_PATH);

    const generateTags = async () => {
        if (projectPath) {
            try {
                // Create a new Command instance
                const command = new Command("ctags", ["-R", "-f", "./tags", projectPath]);

                // Execute the command and capture the output
                const output = await command.execute();

                if (output.code === 0) {
                    console.log("Tags file generated successfully!");
                } else {
                    console.error(`Failed to generate tags. Error: ${output.stderr}`);
                }
            } catch (error) {
                console.error("Failed to execute ctags command:", error);
            }
        }
    };

    return (
        <div>
            <h1>{projectPath}</h1>
            <button onClick={generateTags}>Generate tags</button>
        </div>
    );
}

export default StructureDiagram;

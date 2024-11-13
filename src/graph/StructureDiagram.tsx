import { useEffect, useState } from "react";
import { Command } from "@tauri-apps/api/shell";
import Statics from "../Statics";
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

type TagKey = [number, number]

type ClassType =
    | { type: 'Undiscovered'; value: string }
    | { type: 'Connected'; value: TagKey }
    | { type: 'DataType'; value: number };

interface ProgramTag {
    type: 'Class' | 'Function' | 'Object';
    name: string;
    class?: ClassType;
    parents?: ClassType[];
}

// The entire data structure types

function StructureDiagram() {
    const projectPath = localStorage.getItem(Statics.PROJECT_PATH);
    const [D, setD] = useState("");
    const [progress, setProgress] = useState<Map<string, number>>(new Map());
    const [allFiles, setAllFiles] = useState<Set<string>>(new Set());
    const [allImports, setAllImports] = useState<Map<number, number[]>>(new Map());
    const [allTags, setAllTags] = useState<Map<number, ProgramTag[]>>(new Map());
    const [childrenTable, setChildrenTable] = useState<Map<TagKey, TagKey[]>>(new Map());

    // const [debug, setDebug] = useState<any>();

    // const getTag = (key: TagKey) => {
    //     return (allTags.get(key[0]) as ProgramTag[])[key[1]];
    // }

    useEffect(() => {
        const progress_listen = listen('progress', (event) => {
            let [key, value] = event.payload as [string, number];

            setProgress((prev) => {
                const updatedProgress = new Map(prev);
                updatedProgress.set(key, value);
                return updatedProgress;
            });
        });

        const project_structure_listen = listen('project_structure', (event) => {
            const [AllFiles, ImportsJson, tags_json, ChildrenJson] = event.payload as
                [
                    Set<string>,
                    Map<number, number[]>,
                    Map<number, ProgramTag[]>,
                    Map<String, TagKey[]>
                ];

            // Transform plain JSON objects to appropriate TypeScript data structures if necessary
            setAllFiles(new Set(AllFiles));
            setAllImports(new Map(Object.entries(ImportsJson).map(([k, v]) => [Number(k), v])));
            setAllTags(new Map(Object.entries(tags_json).map(([k, v]) => [Number(k), v])));
            setChildrenTable(new Map(Object.entries(ChildrenJson).map(([k, v]) => {
                const key = JSON.parse(k) as TagKey;
                return [key, v];
            })));
        })

        // Clean up the event listener on component unmount
        return () => {
            progress_listen.then((f) => f());
            project_structure_listen.then((f) => f());
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
                    await invoke('request_project_structure', { projectPath: projectPath, tagsPath: "tags" }).then((s) => console.log(s));
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
            <h3>{projectPath}</h3>
            <button onClick={generateTags}>Generate tags</button>
            <h5>{D}</h5>

            Display progress : {progress}
            {Array.from(progress.entries()).map(([progressType, progressValue]) => (
                <div key={progressType}>
                    <p>{progressType}: {progressValue} %</p>
                    <progress value={progressValue / 100.0} max="1" />
                </div>
            ))}


            {/* Log all the state data */}
            <p>
                <strong>allFiles:</strong> {JSON.stringify(Array.from(allFiles), null, 2)}
            </p>
            <p>
                <strong>allImports:</strong> {JSON.stringify(
                    Array.from(allImports.entries()), null, 2)}
            </p>
            <p>
                <strong>allTags:</strong> {JSON.stringify(
                    Array.from(allTags.entries()), null, 2)}
            </p>
            <p>
                <strong>childrenTable:</strong> {JSON.stringify(
                    Array.from(childrenTable.entries()), null, 2)}
            </p>
        </div>
    );
}
export default StructureDiagram;
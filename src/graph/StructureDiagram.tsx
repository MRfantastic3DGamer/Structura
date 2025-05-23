import { useEffect, useState } from "react";
import { Command } from "@tauri-apps/api/shell";
import Statics from "../Statics";
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import rawHtml from './Graph.html?raw';

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
    const [debugLogs, setDebugLogs] = useState<Map<string, string>>(new Map());
    const [progress, setProgress] = useState<Map<string, number>>(new Map());
    const [allFiles, setAllFiles] = useState<Set<string>>(new Set());
    const [allImports, setAllImports] = useState<Map<number, number[]>>(new Map());
    const [allTags, setAllTags] = useState<Map<number, ProgramTag[]>>(new Map());
    const [childrenTable, setChildrenTable] = useState<Map<TagKey, TagKey[]>>(new Map());
    const [customClasses, setCustomClasses] = useState<Map<number, [string, number][]>>(new Map());
    const [accessibleScopes, setAccessibleScopes] = useState<Map<number, Map<number, [number, number][]>>>(new Map());
    const [scopedConnectable, setScopedConnectable] = useState<Map<number, Map<number, Map<string, any>>>>(new Map());
    const [html, setHtml] = useState<string>("")
    const [debug, setDebug] = useState<string>("");

    interface Node {
        id: string;
        name: string;
        methods: any[];
        objects: any[];
        x: number;
        y: number;
    }

    const [nodes, setNodes] = useState<Node[]>([]);

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

        const intense_data_listen = listen('intense_data', (event) => {
            const { custom_classes, accessible_scopes, scoped_connectable_s } = event.payload as {
                custom_classes: Record<number, [string, number][]>,
                accessible_scopes: Record<number, Record<number, [number, number][]>>,
                scoped_connectable_s: Record<number, Record<number, Record<string, any>>> // Replace any with your custom type if needed
            };

            const customClassesMap = new Map<number, [string, number][]>(
                Object.entries(custom_classes).map(([k, v]) => [Number(k), v])
            );

            const accessibleScopesMap = new Map<number, Map<number, [number, number][]>>(
                Object.entries(accessible_scopes).map(([k, v]) => [
                    Number(k),
                    new Map<number, [number, number][]>(
                        Object.entries(v).map(([innerK, innerV]) => [Number(innerK), innerV])
                    )
                ])
            );

            const scopedConnectableMap = new Map<number, Map<number, Map<string, any>>>(
                Object.entries(scoped_connectable_s).map(([k, v]) => [
                    Number(k),
                    new Map<number, Map<string, any>>(
                        Object.entries(v).map(([innerK, innerV]) => [
                            Number(innerK),
                            new Map<string, any>(Object.entries(innerV))
                        ])
                    )
                ])
            );

            // Set this data into your React state or context
            setCustomClasses(customClassesMap);
            setAccessibleScopes(accessibleScopesMap);
            setScopedConnectable(scopedConnectableMap);
        });

        // Clean up the event listener on component unmount
        return () => {
            progress_listen.then((f) => f());
            project_structure_listen.then((f) => f());
            intense_data_listen.then((f) => f());
        };
    }, []);

    const connect_objects_methods = () => {
        setNodes(prevNodes => {
            const newDebugLogs = new Map();
            const updatedNodes = prevNodes.map(node => ({
                ...node,
                methods: [],
                objects: [],
            }));

            for (const [key, values] of childrenTable.entries()) {
                const [file, index] = key;
                const parentId = `${String(file)}-${String(index)}`;
                const parentNode = updatedNodes.find(node => node.id === parentId);


                newDebugLogs.set(parentId, (newDebugLogs.get(parentId) || "") + "\n✅ Node found");

                for (const [childFile, childIndex] of values) {
                    const allKeys = Array.from(allTags.keys()).join(", ");
                    newDebugLogs.set(
                        parentId,
                        (newDebugLogs.get(parentId) || "") + `\n🗝 Available keys in allTags: ${allKeys}`
                    );

                    const childArray = allTags.get(Number(childFile));

                    if (!childArray) {
                        newDebugLogs.set(
                            parentId,
                            (newDebugLogs.get(parentId) || "") +
                                `\n🔍 allTags has no key: ${childFile} (typeof: ${typeof childFile})`
                        );
                        continue;
                    }

                    const childTag = childArray[childIndex];

                    if (!childTag) {
                        newDebugLogs.set(
                            parentId,
                            (newDebugLogs.get(parentId) || "") +
                                `\n📌 Not found at ${childFile}, ${childIndex} (Array length: ${childArray.length})`
                        );
                        continue;
                    }

                    newDebugLogs.set(parentId, (newDebugLogs.get(parentId) || "") + `\n📌 Found: ${JSON.stringify(childTag)}`);

                    if ("Function" in childTag) {
                        parentNode.methods.push({
                            name: childTag.Function.name,
                            returnType: "void",
                            args: []
                        });
                        newDebugLogs.set(
                            parentId,
                            (newDebugLogs.get(parentId) || "") + `\n📌 Function Added: ${childTag.Function.name}`
                        );
                    } else if ("Object" in childTag) {
                        parentNode.objects.push(childTag);
                        newDebugLogs.set(
                            parentId,
                            (newDebugLogs.get(parentId) || "") + `\n📌 Object Added: ${childTag.Object.name}`
                        );
                    }
                }
            }

            setDebugLogs(newDebugLogs);
            return updatedNodes;
        });
    };

    useEffect(() => {
        if (allTags.size === 0) {
            console.log("⛔ allTags is empty, skipping nodes initialization");
            return;
        }

        console.log("✅ allTags updated, generating nodes");

        setNodes(() => {
            const newNodes = Array.from(allTags.entries()).flatMap(([file, tags]) =>
                tags
                    .filter(tag => "Class" in tag)
                    .map((tag, index) => ({
                        id: `${String(file)}-${String(index)}`,
                        name: tag.Class.name,
                        methods: [],
                        objects: [],
                        x: 300,
                        y: 300,
                    }))
            );

            console.log("✅ Nodes initialized:", newNodes);
            return newNodes;
        });

        connect_objects_methods();
    }, [allTags]);


    const generateTags = async () => {
        if (projectPath) {
            try {
                // Create a new Command instance
                const command = new Command("ctags", ["-R", "--recurse=yes", "-f tags", projectPath]);
                // Execute the command and capture the output
                const output = await command.execute();
                // setD(JSON.stringify(output));
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

    useEffect(() => {
        const newLinks: { source: number; target: number; }[] = [
            // { source: 1, target: 2 },
            // { source: 3, target: 4 }
        ];

        var debug = "";
        nodes.forEach((node, nodeIdx) => {
            const [nf, ni] = node.id.split('-');
            const classTag = allTags.get(Number(nf))[Number(ni)].Class;
            classTag.parents.forEach((p) => {
                if ("Connected" in p) {
                        const [pf, pi] = p.Connected;
                        const pIndex = nodes.findIndex((n) => n.id === `${String(pf)}-${String(pi)}`)
                        newLinks.push({ source: pIndex, target: nodeIdx });
                }
            });
        })
        setDebug(debug);
        const updatedHtmlContent = rawHtml
            .replace("@NODES", JSON.stringify(nodes, null, 2))
            .replace("@LINKS", JSON.stringify(newLinks, null, 2));

        setHtml(updatedHtmlContent);
    }, [nodes]);


    const downloadHtmlFile = () => {
        const blob = new Blob([html], { type: "text/html" });
        const url = URL.createObjectURL(blob);

        const link = document.createElement("a");
        link.href = url;
        link.download = "nodes-data.html";
        link.click();

        URL.revokeObjectURL(url);
    };

    return (
        <div>
            {/* <h3>{projectPath}</h3>
            <button onClick={generateTags}>Generate tags</button>
            <button onClick={connect_objects_methods}>Generate tags</button>
            <h5>{D}</h5>
            <div className="bg-gray-900 text-white p-4 rounded-md overflow-auto max-h-64">
                {Array.from(debugLogs.entries()).map(([key, log]) => (
                    <div key={key} className="border-b border-gray-700 pb-2 mb-2">
                        <strong className="text-blue-400">{key}</strong>
                        <pre className="whitespace-pre-wrap">{log}</pre>
                    </div>
                ))}
            </div> */}

            {/* Display progress : {progress}
            {Array.from(progress.entries()).map(([progressType, progressValue]) => (
                <div key={progressType}>
                    <p>{progressType}: {progressValue} %</p>
                    <progress value={progressValue / 100.0} max="1" />
                </div>
            ))} */}

            {/* <p>
                <strong>allFiles:</strong> {JSON.stringify(Array.from(allFiles), null, 2)}
            </p>
            <p>
                <strong>allImports:</strong> {JSON.stringify(
                    Array.from(allImports.entries()), null, 2)}
            </p> */}
            {/* <p>
                <strong>childrenTable:</strong> {JSON.stringify(
                    Array.from(childrenTable.entries()), null, 2)}
            </p>
            <p>
                <strong>accessibleScopes:</strong> {JSON.stringify(
                    Array.from(accessibleScopes.entries()), null, 2)}
            </p>
            <p>
                <strong>scopedConnectable:</strong> {JSON.stringify(
                    Array.from(scopedConnectable.entries()), null, 2)}
            </p>
            <p>
                <strong>nodes:</strong> {JSON.stringify(
                    nodes, null, 2)}
            </p> */}


            <div style={{
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                backgroundColor: "#f5f5f5",
                padding: "10px",
                boxShadow: "0px 2px 5px rgba(0, 0, 0, 0.1)",
                zIndex: 10
            }}>
                <h3 style={{ display: "inline-block", marginRight: "20px" }}>{projectPath}</h3>
                <button onClick={generateTags} style={{ marginRight: "10px", fontSize: "16px" }} title="Generate Tags">⚙️</button>
                <button onClick={connect_objects_methods} style={{ marginRight: "10px", fontSize: "16px" }} title="Connect Objects and Methods">🔗</button>
                <button onClick={downloadHtmlFile} style={{ fontSize: "16px" }} title="Download HTML">⬇️</button>
            </div>

            <iframe
                srcDoc={html}
                style={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    width: "100%",
                    height: "100%",
                    zIndex: 0,
                    border: "none",
                    backgroundColor: "#ffffff"
                }}
                title="Rendered HTML"
            />

            {/* <p>
                <strong>allTags:</strong> {JSON.stringify(Array.from(allTags.entries()), null, 2)}
            </p>

            <p>
                <strong>Debug:</strong> {debug}
            </p> */}
        </div>
    );
}
export default StructureDiagram;
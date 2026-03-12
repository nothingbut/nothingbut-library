import { writable } from 'svelte/store';

export interface Workspace {
	id: string;
	name: string;
	path: string;
	createdAt: Date;
}

export const currentWorkspace = writable<Workspace | null>(null);
export const workspaceList = writable<Workspace[]>([]);

export function setCurrentWorkspace(workspace: Workspace | null) {
	currentWorkspace.set(workspace);
}

export function setWorkspaceList(workspaces: Workspace[]) {
	workspaceList.set(workspaces);
}

export function addWorkspace(workspace: Workspace) {
	workspaceList.update((list) => [...list, workspace]);
}

export function removeWorkspace(workspaceId: string) {
	workspaceList.update((list) => list.filter((ws) => ws.id !== workspaceId));
}

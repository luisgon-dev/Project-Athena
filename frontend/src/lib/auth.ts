import { writable } from 'svelte/store';
import { getAuthBootstrap, getCurrentUser, login, logout, setupAuth } from '$lib/api';
import type { AuthBootstrapStatus } from '$lib/types/AuthBootstrapStatus';
import type { LoginRequest } from '$lib/types/LoginRequest';
import type { SetupRequest } from '$lib/types/SetupRequest';

export type AuthState = {
	loading: boolean;
	setupRequired: boolean;
	user: AuthBootstrapStatus['authenticated_user'];
};

const initialState: AuthState = {
	loading: true,
	setupRequired: false,
	user: null
};

export const authState = writable<AuthState>(initialState);

export async function refreshAuth(): Promise<AuthState> {
	const status = await getAuthBootstrap();
	const nextState: AuthState = {
		loading: false,
		setupRequired: status.setup_required,
		user: status.authenticated_user
	};
	authState.set(nextState);
	return nextState;
}

export async function loginAndRefresh(payload: LoginRequest) {
	await login(payload);
	const user = await getCurrentUser();
	authState.set({ loading: false, setupRequired: false, user });
	return user;
}

export async function setupAndRefresh(payload: SetupRequest) {
	await setupAuth(payload);
	const user = await getCurrentUser();
	authState.set({ loading: false, setupRequired: false, user });
	return user;
}

export async function logoutAndReset() {
	await logout();
	authState.set({ loading: false, setupRequired: false, user: null });
}

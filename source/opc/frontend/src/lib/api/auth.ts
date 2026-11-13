import { auth as authClient } from '@/integrations/api/client';

export async function getCurrentUser() {
    try {
        const user = await authClient.getCurrentUser();
        return {
            id: user.id,
            email: user.email,
            user_metadata: {
                name: user.name,
                avatar_url: user.avatar_url,
            },
        };
    } catch {
        return null;
    }
}

export async function isAuthenticated(): Promise<boolean> {
    try {
        await authClient.getCurrentUser();
        return true;
    } catch {
        return false;
    }
}

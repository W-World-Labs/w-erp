/**
 * Auth Context - Uses W-ID SSO OAuth via local Rust backend
 */
import React, { createContext, useContext, useEffect, useState } from 'react';
import { auth } from '@/integrations/api/client';

interface User {
  id: string;
  email: string;
  name?: string;
  avatar_url?: string;
}

interface AuthContextType {
  user: User | null;
  loading: boolean;
  isAdmin: boolean;
  signIn: () => void;
  signOut: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  // All authenticated users in this single-tenant system are admins
  const isAdmin = !!user;

  useEffect(() => {
    // /auth/me is the source of truth: the HttpOnly cookie is intentionally
    // invisible to JavaScript.
    auth.getCurrentUser()
      .then(setUser)
      .catch(() => setUser(null))
      .finally(() => setLoading(false));
  }, []);

  const signIn = () => {
    auth.login();
  };

  const signOut = () => {
    auth.logout();
  };

  return (
    <AuthContext.Provider
      value={{
        user,
        loading,
        isAdmin,
        signIn,
        signOut,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}

export interface VaultIdentity {
  app_version: string;
  build_revision: string;
  built_at: string;
  build_profile: string;
  isolated_storage: boolean;
  database_path: string;
}
export default function BuildInfo({ identity }: { identity: VaultIdentity }) {
  const built = new Date(Number(identity.built_at) * 1000);
  return (
    <details className="build-info">
      <summary>
        Pre-Drink {identity.app_version} · {identity.build_profile} ·{" "}
        {identity.isolated_storage
          ? "Separate test storage"
          : "Normal vault location"}
      </summary>
      <p className="fine-print">
        Build {identity.build_revision} · {built.toLocaleString()}
      </p>
      <p className="fine-print">
        {identity.isolated_storage
          ? "This development app uses a separate storage folder. Its vault and password are independent of your normal vault."
          : "This app opens the normal vault for your Mac account. Use the password you chose for this vault."}
      </p>
      <code className="database-path">{identity.database_path}</code>
    </details>
  );
}

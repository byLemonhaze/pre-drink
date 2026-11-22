import type { PrivacyUpdate } from "./types";

export default function PrivacyNotice({ update }: { update: PrivacyUpdate }) {
  if (!update.flagged && !update.approvals_reset) return null;
  return (
    <div className="preview-banner" role="status">
      {update.legacy_review_upgrade ? (
        <>
          Your vault was upgraded to saved-version review.{" "}
          {update.approvals_reset} earlier approvals returned to draft and need
          a fresh review before sharing. {update.flagged} new privacy
          restrictions were added.
        </>
      ) : (
        <>
          Privacy rules added {update.flagged} restrictions when this vault
          opened. {update.approvals_reset} linked approvals returned to draft.
          Unrelated approvals were preserved.
        </>
      )}{" "}
      Review sensitive flags in Settings.
    </div>
  );
}

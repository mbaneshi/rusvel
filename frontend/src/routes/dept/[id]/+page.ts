import { redirect } from '@sveltejs/kit';

export const load = ({ params }: { params: { id: string } }) => {
	throw redirect(302, `/dept/${encodeURIComponent(params.id)}/actions`);
};

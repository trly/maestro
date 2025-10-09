import { y as ensure_array_like, z as attr, F as attr_class } from "../../chunks/index.js";
import "@sourcegraph/amp-sdk";
import { e as escape_html } from "../../chunks/context.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let repositories = [""];
    let prompt = "";
    let results = [];
    let isRunning = false;
    $$renderer2.push(`<div class="min-h-screen bg-gray-50 p-8"><div class="max-w-6xl mx-auto"><h1 class="text-4xl font-bold mb-8 text-gray-900">Maestro</h1> <div class="bg-white rounded-lg shadow-md p-6 mb-6"><h2 class="text-2xl font-semibold mb-4 text-gray-800">Repositories</h2> <!--[-->`);
    const each_array = ensure_array_like(repositories);
    for (let i = 0, $$length = each_array.length; i < $$length; i++) {
      each_array[i];
      $$renderer2.push(`<div class="flex gap-2 mb-3"><input type="text"${attr("value", repositories[i])} placeholder="Repository URL (e.g., github.com/user/repo)" class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"/> `);
      if (repositories.length > 1) {
        $$renderer2.push("<!--[-->");
        $$renderer2.push(`<button class="px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors">Remove</button>`);
      } else {
        $$renderer2.push("<!--[!-->");
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]--> <button class="mt-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors">+ Add Repository</button></div> <div class="bg-white rounded-lg shadow-md p-6 mb-6"><h2 class="text-2xl font-semibold mb-4 text-gray-800">Prompt</h2> <textarea placeholder="Enter your prompt to execute across all repositories..." rows="6" class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-y">`);
    const $$body = escape_html(prompt);
    if ($$body) {
      $$renderer2.push(`${$$body}`);
    }
    $$renderer2.push(`</textarea> <button${attr("disabled", isRunning, true)} class="mt-4 px-6 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:bg-gray-400 disabled:cursor-not-allowed font-semibold">${escape_html("Execute Across All Repos")}</button></div> `);
    if (results.length > 0) {
      $$renderer2.push("<!--[-->");
      $$renderer2.push(`<div class="bg-white rounded-lg shadow-md p-6"><h2 class="text-2xl font-semibold mb-4 text-gray-800">Results</h2> <!--[-->`);
      const each_array_1 = ensure_array_like(results);
      for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
        let result = each_array_1[$$index_1];
        $$renderer2.push(`<div class="mb-6 pb-6 border-b border-gray-200 last:border-b-0"><div class="flex items-center gap-3 mb-2"><h3 class="text-lg font-semibold text-gray-700">${escape_html(result.repo)}</h3> <span${attr_class(`px-3 py-1 rounded-full text-sm font-medium ${result.status === "completed" ? "bg-green-100 text-green-800" : result.status === "failed" ? "bg-red-100 text-red-800" : "bg-blue-100 text-blue-800"}`)}>${escape_html(result.status)}</span></div> <pre class="bg-gray-50 p-4 rounded-lg overflow-x-auto text-sm">${escape_html(result.output || "Waiting...")}</pre></div>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    } else {
      $$renderer2.push("<!--[!-->");
    }
    $$renderer2.push(`<!--]--></div></div>`);
  });
}
export {
  _page as default
};

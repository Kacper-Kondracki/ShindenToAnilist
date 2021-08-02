using System.Threading.Tasks;
using AngleSharp;
using AngleSharp.Dom;

namespace ShindenToAnilist
{
    public class HtmlContext : IHtmlContext
    {
        private readonly IBrowsingContext _context;

        public HtmlContext(IBrowsingContext context)
        {
            _context = context;
        }

        public async Task<IDocument> GetDocumentAsync(string url)
        {
            return await _context.OpenAsync(url);
        }
    }
}
using System.Threading.Tasks;
using AngleSharp.Dom;

namespace ShindenToAnilist
{
    public interface IHtmlContext
    {
        Task<IDocument> GetDocumentAsync(string url);
    }
}
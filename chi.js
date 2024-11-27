// For code highlighter for the χ language
var ChiGrammar = (() => {
  "use strict";
  return (e) => {
    const n = {
      keyword: [
        "case",
        "of",
        "𝜆",
        "λ",
        "cons",
        "const",
        "branch",
        "var",
        "lambda",
        "rec",
        "apply",
      ],
      literal: ["True", "False", "Suc", "Zero", "nil"],
    };

    return {
      name: "chi",
      keywords: n,
    };
  };
})();

export default ChiGrammar;

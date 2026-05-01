import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { CoverArt } from "./CoverArt";

describe("CoverArt", () => {
  it("renders image when coverPath exists", () => {
    render(<CoverArt coverPath="/covers/a.jpg" title="Album" />);
    expect(screen.getByRole("img", { name: "Album" })).toHaveAttribute("src", "/covers/a.jpg");
  });

  it("renders stable placeholder when coverPath is missing", () => {
    render(<CoverArt coverPath={null} title="Album" seed="42" />);
    expect(screen.getByLabelText("Album")).toHaveClass("cover-art--placeholder");
  });
});

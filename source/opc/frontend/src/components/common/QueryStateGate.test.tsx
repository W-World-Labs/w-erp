import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { QueryStateGate } from "./QueryStateGate";

describe("QueryStateGate", () => {
  it("renders children when healthy", () => {
    render(
      <QueryStateGate isLoading={false} isError={false}>
        <div>page content</div>
      </QueryStateGate>,
    );
    expect(screen.getByText("page content")).toBeInTheDocument();
    expect(screen.queryByTestId("query-state-error")).not.toBeInTheDocument();
  });

  it("shows error UI with retry before empty/children content", () => {
    const refetch = vi.fn();
    render(
      <QueryStateGate isLoading={false} isError refetch={refetch}>
        <div>empty state should not show</div>
      </QueryStateGate>,
    );

    expect(screen.getByTestId("query-state-error")).toBeInTheDocument();
    expect(screen.getByText("数据加载失败")).toBeInTheDocument();
    expect(
      screen.queryByText("empty state should not show"),
    ).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /重试/ }));
    expect(refetch).toHaveBeenCalledTimes(1);
  });

  it("prefers error over loading", () => {
    render(
      <QueryStateGate isLoading isError refetch={() => undefined}>
        <div>content</div>
      </QueryStateGate>,
    );
    expect(screen.getByTestId("query-state-error")).toBeInTheDocument();
    expect(screen.queryByTestId("query-state-loading")).not.toBeInTheDocument();
  });

  it("renders loading fallback when loading", () => {
    render(
      <QueryStateGate
        isLoading
        isError={false}
        loadingFallback={<div>custom skeleton</div>}
      >
        <div>content</div>
      </QueryStateGate>,
    );
    expect(screen.getByText("custom skeleton")).toBeInTheDocument();
    expect(screen.queryByText("content")).not.toBeInTheDocument();
  });
});

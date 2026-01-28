"use client";

import { useState } from "react";
import { ChevronDown } from "lucide-react";
import { cn } from "@/lib/utils";

const faqs = [
  {
    question: "What file formats are supported?",
    answer:
      "Track Leader currently supports GPX files, which can be exported from most GPS devices and fitness apps including Garmin, Strava, and Apple Watch.",
  },
  {
    question: "How are segments matched to activities?",
    answer:
      "When you upload an activity, our algorithm compares your track against all segments. If your route passes through a segment's start and end points within tolerance, an effort is recorded automatically.",
  },
  {
    question: "What are KOMs and QOMs?",
    answer:
      "KOM (King of the Mountain) and QOM (Queen of the Mountain) are titles given to the fastest male and female athlete on a segment. These are displayed on leaderboards and count toward your crown total.",
  },
  {
    question: "How do demographic filters work?",
    answer:
      "You can optionally set your age and gender in your profile. Leaderboards can then be filtered to show rankings within specific age groups or genders, helping you compare against similar athletes.",
  },
  {
    question: "Can I make my activities private?",
    answer:
      "Yes! Each activity can be set to public or private. Private activities still match segments and appear on leaderboards, but won't show up in other users' feeds.",
  },
  {
    question: "How do I create a segment?",
    answer:
      "Open any of your activities, click 'Create Segment', then click on the elevation profile to select start and end points. Give it a name and submit. The segment will be available for everyone to compete on.",
  },
  {
    question: "Is Track Leader free?",
    answer:
      "Yes, Track Leader is completely free to use. We believe in open competition and community-driven leaderboards without paywalls.",
  },
  {
    question: "How accurate is segment timing?",
    answer:
      "Timing accuracy depends on your GPS device's recording frequency. We use the timestamps from your GPX file to calculate elapsed time between segment start and end points.",
  },
];

export function FAQ() {
  const [openIndex, setOpenIndex] = useState<number | null>(null);

  return (
    <section className="py-16" aria-labelledby="faq-heading">
      <div className="text-center mb-12">
        <h2 id="faq-heading" className="text-3xl font-bold tracking-tight">
          Frequently Asked Questions
        </h2>
        <p className="mt-4 text-lg text-muted-foreground max-w-2xl mx-auto">
          Everything you need to know about Track Leader.
        </p>
      </div>
      <div className="max-w-3xl mx-auto">
        <dl className="space-y-4">
          {faqs.map((faq, index) => (
            <div
              key={faq.question}
              className="rounded-lg border bg-card overflow-hidden"
            >
              <dt>
                <button
                  className="flex w-full items-center justify-between p-4 text-left font-medium hover:bg-muted/50 transition-colors"
                  onClick={() => setOpenIndex(openIndex === index ? null : index)}
                  aria-expanded={openIndex === index}
                  aria-controls={`faq-answer-${index}`}
                >
                  {faq.question}
                  <ChevronDown
                    className={cn(
                      "h-5 w-5 text-muted-foreground transition-transform",
                      openIndex === index && "rotate-180"
                    )}
                    aria-hidden="true"
                  />
                </button>
              </dt>
              <dd
                id={`faq-answer-${index}`}
                className={cn(
                  "grid transition-all duration-200 ease-in-out",
                  openIndex === index
                    ? "grid-rows-[1fr] opacity-100"
                    : "grid-rows-[0fr] opacity-0"
                )}
              >
                <div className="overflow-hidden">
                  <p className="p-4 pt-0 text-muted-foreground">{faq.answer}</p>
                </div>
              </dd>
            </div>
          ))}
        </dl>
      </div>
    </section>
  );
}

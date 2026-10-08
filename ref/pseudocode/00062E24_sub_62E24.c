/* ea:   0x00062E24 */
/* name: sub_62E24 */
/* size: 68 */
void __fastcall sub_62E24(int a1, _DWORD *a2, int a3)
{
  struct CRITICAL_SECTION *v5; // r4

  *a2 = a3;
  v5 = (struct CRITICAL_SECTION *)(a1 + 48);
  a2[1] = MEMORY[0xFFFFC808];
  EnterCriticalSection((LPCRITICAL_SECTION)(a1 + 48));
  a2[2] = *(_DWORD *)(a1 + 92);
  *(_DWORD *)(a1 + 92) = a2;
  LeaveCriticalSection(v5);
}


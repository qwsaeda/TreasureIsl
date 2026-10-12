/* ea:   0x00052E50 */
/* name: sub_52E50 */
/* size: 100 */
int __fastcall sub_52E50(int a1, void *a2)
{
  if ( a2 == (void *)-1 )
    return 0;
  sub_52EB4((LPCVOID)a1, a2);
  sub_52EB4((LPCVOID)(a1 + 4), a2);
  sub_52F18((LPCVOID)(a1 + 8), a2);
  sub_52F7C(a1 + 16, a2);
  sub_53034(a1 + 28, a2);
  sub_530D4((LPCVOID)(a1 + 40), a2);
  return 1;
}


/* ea:   0x000587D8 */
/* name: sub_587D8 */
/* size: 48 */
int __fastcall sub_587D8(int a1, int a2)
{
  sub_5879C(a1);
  if ( a2 == 0 )
    return 0;
  *(_DWORD *)(a1 + 4) = a2;
  *(_BYTE *)(a1 + 28) = 0;
  return 1;
}


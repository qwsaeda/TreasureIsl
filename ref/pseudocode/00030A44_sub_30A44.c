/* ea:   0x00030A44 */
/* name: sub_30A44 */
/* size: 36 */
int __fastcall sub_30A44(int a1, int a2)
{
  unsigned int v3; // r2
  int result; // r0

  v3 = *(_DWORD *)(a1 + 8) + a2;
  result = 999999;
  if ( v3 <= 0xF423F )
    *(_DWORD *)(a1 + 8) = v3;
  else
    *(_DWORD *)(a1 + 8) = 999999;
  return result;
}


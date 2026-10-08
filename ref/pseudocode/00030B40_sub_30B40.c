/* ea:   0x00030B40 */
/* name: sub_30B40 */
/* size: 56 */
unsigned int __fastcall sub_30B40(int a1, unsigned __int16 a2)
{
  unsigned int v2; // r1

  v2 = *(unsigned __int16 *)(a1 + 16) + a2;
  if ( v2 > 9 )
    v2 = 9;
  *(_WORD *)(a1 + 16) = v2;
  return v2 << 16;
}


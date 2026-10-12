/* ea:   0x00030AD0 */
/* name: sub_30AD0 */
/* size: 64 */
int __fastcall sub_30AD0(int a1, unsigned __int16 a2)
{
  unsigned int v3; // r1
  int v4; // r3

  v3 = *(unsigned __int16 *)(a1 + 14) + a2;
  v4 = 999;
  if ( v3 <= 0x3E7 )
    v4 = v3;
  *(_WORD *)(a1 + 14) = v4;
  return v4 << 16;
}


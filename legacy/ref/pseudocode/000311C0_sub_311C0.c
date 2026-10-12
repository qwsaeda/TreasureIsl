/* ea:   0x000311C0 */
/* name: sub_311C0 */
/* size: 64 */
int __fastcall sub_311C0(int a1, unsigned __int16 a2)
{
  unsigned int v3; // r1
  int v4; // r3

  v3 = *(unsigned __int16 *)(a1 + 22) + a2;
  v4 = 999;
  if ( v3 <= 0x3E7 )
    v4 = v3;
  *(_WORD *)(a1 + 22) = v4;
  return v4 << 16;
}


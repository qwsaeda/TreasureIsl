/* ea:   0x00031200 */
/* name: sub_31200 */
/* size: 88 */
int __fastcall sub_31200(int a1, unsigned __int16 a2)
{
  int v3; // r4
  unsigned int v4; // r3
  unsigned __int16 v5; // r0
  unsigned __int16 v6; // r3
  int result; // r0

  v3 = *(unsigned __int16 *)(a1 + 24);
  v4 = *(unsigned __int16 *)(a1 + 22);
  v5 = 0;
  if ( a2 <= v4 )
    v5 = v4 - a2;
  v6 = v5;
  result = v3 << 16;
  *(_WORD *)(a1 + 22) = v6;
  if ( (unsigned __int16)v3 <= (unsigned int)v6 )
    *(_WORD *)(a1 + 24) = v3;
  else
    *(_WORD *)(a1 + 24) = v6;
  return result;
}


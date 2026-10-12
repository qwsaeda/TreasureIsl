/* ea:   0x0004A71C */
/* name: sub_4A71C */
/* size: 60 */
unsigned int __fastcall sub_4A71C(int a1, unsigned __int16 a2)
{
  unsigned int v3; // r3
  unsigned int v4; // r0
  unsigned int result; // r0

  v3 = *(unsigned __int16 *)(a1 + 12);
  v4 = 0;
  if ( a2 <= v3 )
    v4 = v3 - a2;
  result = v4 << 16;
  *(_WORD *)(a1 + 12) = HIWORD(result);
  return result;
}


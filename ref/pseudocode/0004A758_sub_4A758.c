/* ea:   0x0004A758 */
/* name: sub_4A758 */
/* size: 60 */
unsigned int __fastcall sub_4A758(int a1, unsigned __int16 a2)
{
  unsigned int v3; // r3
  unsigned int v4; // r0
  unsigned int result; // r0

  v3 = *(unsigned __int16 *)(a1 + 14);
  v4 = 0;
  if ( a2 <= v3 )
    v4 = v3 - a2;
  result = v4 << 16;
  *(_WORD *)(a1 + 14) = HIWORD(result);
  return result;
}


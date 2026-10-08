/* ea:   0x00053940 */
/* name: sub_53940 */
/* size: 76 */
unsigned int __fastcall sub_53940(int a1, int a2)
{
  unsigned int v3; // r2

  if ( a2 == 0 )
    return 0;
  v3 = ((unsigned int)(*(unsigned __int16 *)(a2 + 24) << 24) >> 30) + 1;
  if ( (*(_BYTE *)(a2 + 22) & 0x3F) != 5 || ((unsigned int)(*(unsigned __int16 *)(a2 + 24) << 24) >> 30) + 1 > 4 )
    return 0;
  return v3;
}


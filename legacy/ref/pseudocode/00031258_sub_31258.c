/* ea:   0x00031258 */
/* name: sub_31258 */
/* size: 68 */
unsigned int __fastcall sub_31258(int a1, unsigned __int16 a2)
{
  unsigned int v3; // r2

  v3 = *(unsigned __int16 *)(a1 + 22);
  if ( *(unsigned __int16 *)(a1 + 24) + (unsigned int)a2 <= v3 )
    v3 = *(unsigned __int16 *)(a1 + 24) + a2;
  *(_WORD *)(a1 + 24) = v3;
  return v3 << 16;
}


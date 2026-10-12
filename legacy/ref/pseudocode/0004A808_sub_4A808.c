/* ea:   0x0004A808 */
/* name: sub_4A808 */
/* size: 148 */
int __fastcall sub_4A808(int a1, unsigned __int16 a2, unsigned __int16 *a3, _WORD *a4)
{
  unsigned __int16 v6; // r3
  __int16 v7; // r0
  __int16 v8; // r2
  int result; // r0

  v6 = *(_WORD *)(a1 + 22);
  if ( a2 <= (unsigned int)v6 )
    v6 = a2;
  *a3 = v6;
  v7 = *(_WORD *)(a1 + 22) - v6;
  *(_WORD *)(a1 + 22) = v7;
  v8 = *(_WORD *)(a1 + 24) - v7;
  *a4 = v8;
  if ( v8 != 0 )
  {
    result = *(unsigned __int16 *)(a1 + 22);
    *(_WORD *)(a1 + 24) = result;
  }
  else
  {
    *a4 = 0;
    *(_WORD *)(a1 + 24) = 0;
    return 0;
  }
  return result;
}


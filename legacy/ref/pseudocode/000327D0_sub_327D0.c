/* ea:   0x000327D0 */
/* name: sub_327D0 */
/* size: 204 */
BOOL __fastcall sub_327D0(int a1, int a2)
{
  int v3; // r7
  int v4; // r6
  unsigned __int16 *v5; // r5
  int v6; // r4
  signed int v7; // r0

  if ( a2 == 0 )
    return false;
  v3 = 1;
  v4 = 2 * (2 * *(unsigned __int16 *)(a2 + 24) + *(unsigned __int16 *)(a2 + 22)) * *(unsigned __int8 *)(a2 + 49);
  v5 = (unsigned __int16 *)(a1 + 136);
  v6 = sub_16C9C((unsigned __int16 *)(a1 + 136), 0);
  v7 = (unsigned int)(v6 + 2 * sub_16C9C(v5, 1)) >> 1;
  if ( v7 != 0 && v4 / v7 == 0 )
    return rand() % 100 >= 100;
  return v3;
}


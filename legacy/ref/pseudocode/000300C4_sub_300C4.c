/* ea:   0x000300C4 */
/* name: sub_300C4 */
/* size: 104 */
int __fastcall sub_300C4(_DWORD *a1, unsigned int a2, int a3)
{
  unsigned int v4; // r1

  v4 = (a1[1] - *a1) / 12;
  if ( a2 >= v4 )
    return sub_35818(a1, a1[1], a2 - v4, a3);
  else
    return sub_32478((int)a1, *a1 + 12 * a2, a1[1]);
}


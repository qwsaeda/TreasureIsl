/* ea:   0x00034890 */
/* name: sub_34890 */
/* size: 104 */
int __fastcall sub_34890(_DWORD *a1, unsigned int a2, int a3)
{
  unsigned int v4; // r1

  v4 = (a1[1] - *a1) / 12;
  if ( a2 >= v4 )
    return sub_36C50(a1, a1[1], a2 - v4, a3);
  else
    return sub_34950(a1, *a1 + 12 * a2, a1[1]);
}


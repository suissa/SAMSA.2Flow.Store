package net.sansa_stack.rdf.spark.io

import org.apache.jena.datatypes.xsd.XSDDatatype
import org.apache.jena.graph.{Node, NodeFactory, Triple}
import org.apache.spark.rdd.RDD
import org.apache.spark.sql.{Dataset, Encoders, SparkSession}

/**
 * Reader for the 2flow graph notation.
 *
 * Each non-empty, non-comment line must follow this syntax:
 * {{
 *   Subject -> predicate:Object
 * }}
 *
 * Terms are expanded against a base URI, so `Torre_Eiffel -> altura:330` becomes:
 * {{
 *   <http://2flow.org/Torre_Eiffel> <http://2flow.org/altura> "330"^^xsd:integer
 * }}
 */
object FlowRDFReader extends Serializable {
  val DefaultBaseUri = "http://2flow.org"

  private val FlowLine = """^\s*([^\s]+)\s*->\s*([^:\s]+)\s*:\s*(.+?)\s*$""".r
  private val IntegerValue = """^[+-]?\d+$""".r
  private val DecimalValue = """^[+-]?(?:\d+\.\d*|\d*\.\d+)$""".r

  /** Load a 2flow text file as an RDD of Apache Jena triples. */
  def load(spark: SparkSession, path: String, baseUri: String = DefaultBaseUri): RDD[Triple] = {
    parseLines(spark.sparkContext.textFile(path), baseUri)
  }

  /** Convert an RDD of 2flow lines into an RDD of Apache Jena triples. */
  def parseLines(lines: RDD[String], baseUri: String = DefaultBaseUri): RDD[Triple] = {
    val normalizedBaseUri = normalizeBaseUri(baseUri)
    lines
      .map(_.trim)
      .filter(line => line.nonEmpty && !line.startsWith("#"))
      .map(parseLine(_, normalizedBaseUri))
  }

  /** Load a 2flow text file as a Dataset of Apache Jena triples. */
  def loadDataset(spark: SparkSession, path: String, baseUri: String = DefaultBaseUri): Dataset[Triple] = {
    spark.createDataset(load(spark, path, baseUri))(Encoders.kryo[Triple])
  }

  /** Convert a single 2flow line into an Apache Jena triple. */
  def parseLine(line: String, baseUri: String = DefaultBaseUri): Triple = line match {
    case FlowLine(subject, predicate, obj) =>
      Triple.create(
        termToUri(subject, baseUri),
        termToUri(predicate, baseUri),
        objectToNode(obj.trim, baseUri))
    case _ =>
      throw new IllegalArgumentException(
        s"Invalid 2flow line '$line'. Expected syntax: Subject -> predicate:Object")
  }

  private def objectToNode(value: String, baseUri: String): Node = value match {
    case IntegerValue() => NodeFactory.createLiteral(value, XSDDatatype.XSDinteger)
    case DecimalValue() => NodeFactory.createLiteral(value, XSDDatatype.XSDdecimal)
    case _ => termToUri(value, baseUri)
  }

  private def termToUri(term: String, baseUri: String): Node = {
    NodeFactory.createURI(normalizeBaseUri(baseUri) + encodeTerm(term.trim))
  }

  private def normalizeBaseUri(baseUri: String): String = {
    if (baseUri.endsWith("/") || baseUri.endsWith("#")) baseUri else baseUri + "/"
  }

  private def encodeTerm(term: String): String = {
    java.net.URLEncoder.encode(term, "UTF-8").replace("+", "%20")
  }
}
